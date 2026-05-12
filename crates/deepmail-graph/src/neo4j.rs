use crate::error::GraphError;
use neo4rs::{query, Graph};
use std::collections::HashMap;
use std::sync::Arc;

pub const ALLOWED_LABELS: &[&str] = &[
    "Email",
    "IpAddress",
    "Domain",
    "Url",
    "FileHash",
    "Sender",
    "Campaign",
];

pub const ALLOWED_REL_TYPES: &[&str] = &[
    "CONTAINS_IOC",
    "RESOLVES_TO",
    "HOSTED_ON",
    "SUBDOMAIN_OF",
    "SENT_BY",
    "HAS_HASH",
    "C2_COMMUNICATES_WITH",
    "MEMBER_OF",
];

fn validate_label(label: &str) -> Result<(), GraphError> {
    if ALLOWED_LABELS.contains(&label) {
        Ok(())
    } else {
        Err(GraphError::InvalidLabel(label.to_string()))
    }
}

fn validate_rel_type(rel_type: &str) -> Result<(), GraphError> {
    if ALLOWED_REL_TYPES.contains(&rel_type) {
        Ok(())
    } else {
        Err(GraphError::InvalidRelType(rel_type.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct GraphNodeData {
    pub neo4j_id: String,
    pub node_type: String,
    pub value: String,
    pub threat_score: f32,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct GraphEdgeData {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub confidence: f32,
}

#[derive(Clone)]
pub struct Neo4jClient {
    graph: Arc<Graph>,
}

impl Neo4jClient {
    pub async fn connect(uri: &str, user: &str, password: &str) -> Result<Self, GraphError> {
        let host = uri
            .strip_prefix("bolt://")
            .or_else(|| uri.strip_prefix("neo4j://"))
            .unwrap_or(uri);
        let graph = Graph::new(host, user, password)
            .await
            .map_err(|e| GraphError::Neo4jConnection(e.to_string()))?;
        Ok(Self {
            graph: Arc::new(graph),
        })
    }

    pub async fn merge_node(
        &self,
        label: &str,
        value: &str,
        properties: &HashMap<String, String>,
    ) -> Result<String, GraphError> {
        validate_label(label)?;

        let entries: Vec<(&String, &String)> = properties.iter().collect();

        let set_parts: Vec<String> = entries
            .iter()
            .enumerate()
            .map(|(i, (k, _))| format!("n.`{k}` = $prop_{i}"))
            .collect();

        let set_clause = if set_parts.is_empty() {
            String::new()
        } else {
            format!("SET {}", set_parts.join(", "))
        };

        let cypher = format!(
            "MERGE (n:{label} {{value: $value}}) {set_clause} RETURN id(n) AS neo4j_id"
        );

        let mut q = query(&cypher).param("value", value);
        for (i, (_, v)) in entries.iter().enumerate() {
            q = q.param(&format!("prop_{i}"), v.as_str());
        }

        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        match result.next().await {
            Ok(Some(row)) => {
                let id: i64 = row
                    .get("neo4j_id")
                    .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;
                Ok(id.to_string())
            }
            Ok(None) => Err(GraphError::Neo4jQuery("MERGE returned no rows".into())),
            Err(e) => Err(GraphError::Neo4jQuery(e.to_string())),
        }
    }

    pub async fn merge_relationship(
        &self,
        from_label: &str,
        from_value: &str,
        to_label: &str,
        to_value: &str,
        rel_type: &str,
        props: &HashMap<String, String>,
    ) -> Result<(), GraphError> {
        validate_label(from_label)?;
        validate_label(to_label)?;
        validate_rel_type(rel_type)?;

        let entries: Vec<(&String, &String)> = props.iter().collect();

        let set_parts: Vec<String> = entries
            .iter()
            .enumerate()
            .map(|(i, (k, _))| format!("r.`{k}` = $rp_{i}"))
            .collect();

        let set_clause = if set_parts.is_empty() {
            String::new()
        } else {
            format!("SET {}", set_parts.join(", "))
        };

        let cypher = format!(
            "MATCH (a:{from_label} {{value: $from_value}}), (b:{to_label} {{value: $to_value}}) \
             MERGE (a)-[r:{rel_type}]->(b) {set_clause} RETURN type(r) AS rtype"
        );

        let mut q = query(&cypher)
            .param("from_value", from_value)
            .param("to_value", to_value);

        for (i, (_, v)) in entries.iter().enumerate() {
            q = q.param(&format!("rp_{i}"), v.as_str());
        }

        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let _ = result.next().await;
        Ok(())
    }

    pub async fn query_related(
        &self,
        value: &str,
        label: &str,
        depth: i32,
    ) -> Result<(Vec<GraphNodeData>, Vec<GraphEdgeData>), GraphError> {
        validate_label(label)?;
        let depth = depth.clamp(1, 4);

        let node_cypher = format!(
            "MATCH (start:{label} {{value: $value}}) \
             OPTIONAL MATCH (start)-[*1..{depth}]-(n) \
             WITH [start] + collect(DISTINCT n) AS all_nodes \
             UNWIND all_nodes AS node \
             WITH DISTINCT node WHERE node IS NOT NULL \
             RETURN id(node) AS neo4j_id, labels(node)[0] AS node_type, \
                    node.value AS value, coalesce(node.threat_score, 0.0) AS threat_score"
        );

        let q = query(&node_cypher).param("value", value);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let mut nodes = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        while let Ok(Some(row)) = result.next().await {
            let neo4j_id: i64 = row.get("neo4j_id").unwrap_or(0);
            if !seen_ids.insert(neo4j_id) {
                continue;
            }
            let node_type: String = row.get("node_type").unwrap_or_default();
            let val: String = row.get("value").unwrap_or_default();
            let threat_score: f64 = row.get("threat_score").unwrap_or(0.0);

            nodes.push(GraphNodeData {
                neo4j_id: neo4j_id.to_string(),
                node_type,
                value: val,
                threat_score: threat_score as f32,
                properties: HashMap::new(),
            });
        }

        let edge_cypher = format!(
            "MATCH (start:{label} {{value: $value}}) \
             MATCH path = (start)-[*1..{depth}]-(end) \
             UNWIND relationships(path) AS r \
             WITH DISTINCT r \
             RETURN id(startNode(r)) AS source_id, id(endNode(r)) AS target_id, \
                    type(r) AS relation_type, coalesce(r.confidence, 1.0) AS confidence"
        );

        let q = query(&edge_cypher).param("value", value);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let mut edges = Vec::new();
        while let Ok(Some(row)) = result.next().await {
            let source_id: i64 = row.get("source_id").unwrap_or(0);
            let target_id: i64 = row.get("target_id").unwrap_or(0);
            let relation_type: String = row.get("relation_type").unwrap_or_default();
            let confidence: f64 = row.get("confidence").unwrap_or(1.0);

            edges.push(GraphEdgeData {
                source_id: source_id.to_string(),
                target_id: target_id.to_string(),
                relation_type,
                confidence: confidence as f32,
            });
        }

        Ok((nodes, edges))
    }

    pub async fn find_c2_paths(
        &self,
        email_id: &str,
    ) -> Result<Vec<Vec<GraphNodeData>>, GraphError> {
        let cypher = "\
            MATCH (email:Email {value: $email_id})-[:CONTAINS_IOC]->(ioc) \
                  -[:C2_COMMUNICATES_WITH|RESOLVES_TO|HOSTED_ON*1..3]->(target) \
            RETURN id(email) AS e_id, email.value AS e_value, \
                   id(ioc) AS i_id, labels(ioc)[0] AS i_type, ioc.value AS i_value, \
                   coalesce(ioc.threat_score, 0.0) AS i_score, \
                   id(target) AS t_id, labels(target)[0] AS t_type, target.value AS t_value, \
                   coalesce(target.threat_score, 0.0) AS t_score";

        let q = query(cypher).param("email_id", email_id);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let mut paths = Vec::new();
        while let Ok(Some(row)) = result.next().await {
            let e_id: i64 = row.get("e_id").unwrap_or(0);
            let e_value: String = row.get("e_value").unwrap_or_default();
            let i_id: i64 = row.get("i_id").unwrap_or(0);
            let i_type: String = row.get("i_type").unwrap_or_default();
            let i_value: String = row.get("i_value").unwrap_or_default();
            let i_score: f64 = row.get("i_score").unwrap_or(0.0);
            let t_id: i64 = row.get("t_id").unwrap_or(0);
            let t_type: String = row.get("t_type").unwrap_or_default();
            let t_value: String = row.get("t_value").unwrap_or_default();
            let t_score: f64 = row.get("t_score").unwrap_or(0.0);

            paths.push(vec![
                GraphNodeData {
                    neo4j_id: e_id.to_string(),
                    node_type: "Email".to_string(),
                    value: e_value,
                    threat_score: 0.0,
                    properties: HashMap::new(),
                },
                GraphNodeData {
                    neo4j_id: i_id.to_string(),
                    node_type: i_type,
                    value: i_value,
                    threat_score: i_score as f32,
                    properties: HashMap::new(),
                },
                GraphNodeData {
                    neo4j_id: t_id.to_string(),
                    node_type: t_type,
                    value: t_value,
                    threat_score: t_score as f32,
                    properties: HashMap::new(),
                },
            ]);
        }

        Ok(paths)
    }

    pub async fn find_shadow_infra(
        &self,
        ip_value: &str,
    ) -> Result<Vec<GraphNodeData>, GraphError> {
        let cypher = "\
            MATCH (ip:IpAddress {value: $ip_value})-[*1..2]-(related) \
            RETURN DISTINCT id(related) AS neo4j_id, labels(related)[0] AS node_type, \
                   related.value AS value, coalesce(related.threat_score, 0.0) AS threat_score";

        let q = query(cypher).param("ip_value", ip_value);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let mut nodes = Vec::new();
        while let Ok(Some(row)) = result.next().await {
            let neo4j_id: i64 = row.get("neo4j_id").unwrap_or(0);
            let node_type: String = row.get("node_type").unwrap_or_default();
            let val: String = row.get("value").unwrap_or_default();
            let threat_score: f64 = row.get("threat_score").unwrap_or(0.0);

            nodes.push(GraphNodeData {
                neo4j_id: neo4j_id.to_string(),
                node_type,
                value: val,
                threat_score: threat_score as f32,
                properties: HashMap::new(),
            });
        }

        Ok(nodes)
    }

    pub async fn get_campaign_graph(
        &self,
        campaign_id: &str,
    ) -> Result<(Vec<GraphNodeData>, Vec<GraphEdgeData>, i32), GraphError> {
        let count_cypher = "\
            MATCH (c:Campaign {value: $cid})<-[:MEMBER_OF]-(email:Email) \
            RETURN count(DISTINCT email) AS email_count";

        let q = query(count_cypher).param("cid", campaign_id);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let email_count = match result.next().await {
            Ok(Some(row)) => {
                let c: i64 = row.get("email_count").unwrap_or(0);
                c as i32
            }
            _ => 0,
        };

        let node_cypher = "\
            MATCH (c:Campaign {value: $cid})<-[:MEMBER_OF]-(email:Email) \
            OPTIONAL MATCH (email)-[:CONTAINS_IOC]->(ioc) \
            WITH collect(DISTINCT c) + collect(DISTINCT email) + collect(DISTINCT ioc) AS all \
            UNWIND all AS node \
            WITH DISTINCT node WHERE node IS NOT NULL \
            RETURN id(node) AS neo4j_id, labels(node)[0] AS node_type, \
                   node.value AS value, coalesce(node.threat_score, 0.0) AS threat_score";

        let q = query(node_cypher).param("cid", campaign_id);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let mut nodes = Vec::new();
        while let Ok(Some(row)) = result.next().await {
            let neo4j_id: i64 = row.get("neo4j_id").unwrap_or(0);
            let node_type: String = row.get("node_type").unwrap_or_default();
            let val: String = row.get("value").unwrap_or_default();
            let threat_score: f64 = row.get("threat_score").unwrap_or(0.0);

            nodes.push(GraphNodeData {
                neo4j_id: neo4j_id.to_string(),
                node_type,
                value: val,
                threat_score: threat_score as f32,
                properties: HashMap::new(),
            });
        }

        let edge_cypher = "\
            MATCH (c:Campaign {value: $cid})<-[:MEMBER_OF]-(email:Email) \
            OPTIONAL MATCH (email)-[:CONTAINS_IOC]->(ioc) \
            WITH collect(DISTINCT c) + collect(DISTINCT email) + collect(DISTINCT ioc) AS all \
            UNWIND all AS a \
            MATCH (a)-[r]->(b) WHERE b IN all \
            RETURN DISTINCT id(startNode(r)) AS source_id, id(endNode(r)) AS target_id, \
                   type(r) AS relation_type, coalesce(r.confidence, 1.0) AS confidence";

        let q = query(edge_cypher).param("cid", campaign_id);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| GraphError::Neo4jQuery(e.to_string()))?;

        let mut edges = Vec::new();
        while let Ok(Some(row)) = result.next().await {
            let source_id: i64 = row.get("source_id").unwrap_or(0);
            let target_id: i64 = row.get("target_id").unwrap_or(0);
            let relation_type: String = row.get("relation_type").unwrap_or_default();
            let confidence: f64 = row.get("confidence").unwrap_or(1.0);

            edges.push(GraphEdgeData {
                source_id: source_id.to_string(),
                target_id: target_id.to_string(),
                relation_type,
                confidence: confidence as f32,
            });
        }

        Ok((nodes, edges, email_count))
    }
}
