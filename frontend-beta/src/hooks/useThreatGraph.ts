"use client";

import { useState, useCallback, useMemo, useEffect } from "react";
import { EmailAnalysisReport, IocEntry, IocType } from "@/lib/types";

export type NodeType = "email" | "ip" | "domain" | "url" | "file";

export interface GraphNode {
	id: string;
	type: NodeType;
	value: string;
	risk: "Safe" | "Suspicious" | "Critical";
	isMalicious: boolean;
	metadata?: any;
	collapsed?: boolean;
	hidden?: boolean;
	x?: number;
	y?: number;
	vx?: number;
	vy?: number;
	fx?: number | null;
	fy?: number | null;
}

export interface GraphLink {
	source: string;
	target: string;
	type: "contains" | "sent_from" | "resolves_to" | "attached_to";
}

export function useThreatGraph() {
	const [nodes, setNodes] = useState<Record<string, GraphNode>>({});
	const [links, setLinks] = useState<GraphLink[]>([]);
	const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

	const addEmailAnalysis = useCallback((report: EmailAnalysisReport) => {
		const newNodes: Record<string, GraphNode> = {};
		const newLinks: GraphLink[] = [];

		const emailId = report.email.id;
		const riskLevel =
			(report.analysis_results.find((r) => r.threat_score !== undefined)
				?.threat_score || 0) > 70
				? "Critical"
				: "Safe";

		// 1. Add Email Node
		newNodes[emailId] = {
			id: emailId,
			type: "email",
			value: report.email.original_name,
			risk: riskLevel as any,
			isMalicious: riskLevel === "Critical",
			metadata: report.email,
		};

		// 2. Add IOC Nodes and Links
		report.iocs.forEach((ioc) => {
			const typeMap: Record<string, NodeType> = {
				ip: "ip",
				domain: "domain",
				url: "url",
				md5: "file",
				sha1: "file",
				sha256: "file",
				email: "email",
			};

			const nodeType = typeMap[ioc.ioc_type] || "url";
			const nodeId = `${nodeType}:${ioc.value}`;

			newNodes[nodeId] = {
				id: nodeId,
				type: nodeType,
				value: ioc.value,
				risk: "Suspicious", // Default for IOCs in analysis
				isMalicious: false, // Will be updated by scoring
				metadata: ioc,
			};

			// Relationship: email -> contains -> IOC
			newLinks.push({
				source: emailId,
				target: nodeId,
				type: "contains",
			});

			// Specific logic: URL -> resolves_to -> IP (if metadata suggests)
			// This would require backend support or frontend lookup
		});

		setNodes((prev) => ({ ...prev, ...newNodes }));
		setLinks((prev) => [...prev, ...newLinks]);
	}, []);

	const graphData = useMemo(() => {
		return {
			nodes: Object.values(nodes).filter((n) => !n.hidden),
			links: links.filter(
				(l) =>
					nodes[l.source as string] &&
					!nodes[l.source as string].hidden &&
					nodes[l.target as string] &&
					!nodes[l.target as string].hidden,
			),
		};
	}, [nodes, links]);

	const toggleNodeCollapse = useCallback(
		(nodeId: string) => {
			setNodes((prev) => {
				const node = prev[nodeId];
				if (!node) return prev;

				const isCollapsed = !node.collapsed;
				const nextNodes = {
					...prev,
					[nodeId]: { ...node, collapsed: isCollapsed },
				};

				// Find children and hide them if collapsed
				// For simplicity: hide nodes that only have a link from this node
				const childrenIds = links
					.filter(
						(l) =>
							l.source === nodeId ||
							(l.source as any).id === nodeId,
					)
					.map((l) =>
						typeof l.target === "string"
							? l.target
							: (l.target as any).id,
					);

				childrenIds.forEach((childId) => {
					if (nextNodes[childId]) {
						nextNodes[childId] = {
							...nextNodes[childId],
							hidden: isCollapsed,
						};
					}
				});

				return nextNodes;
			});
		},
		[links],
	);

	return {
		nodes,
		links,
		graphData,
		selectedNodeId,
		setSelectedNodeId,
		addEmailAnalysis,
		toggleNodeCollapse,
		selectedNode: selectedNodeId ? nodes[selectedNodeId] : null,
	};
}
