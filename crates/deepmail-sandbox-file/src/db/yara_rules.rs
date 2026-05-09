/// Database operations for yara_rules table.

use sqlx::PgPool;

/// Increment match_count for a list of matched rule names.
pub async fn increment_match_counts(
    pool: &PgPool,
    rule_names: &[String],
) -> Result<(), sqlx::Error> {
    if rule_names.is_empty() {
        return Ok(());
    }

    for name in rule_names {
        sqlx::query(
            "UPDATE yara_rules
             SET match_count = match_count + 1, updated_at = now()
             WHERE rule_name = $1"
        )
        .bind(name)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Ensure default YARA rules are in the database.
#[allow(dead_code)]
pub async fn upsert_default_rules(
    pool: &PgPool,
    rules: &[(&str, &str)],
) -> Result<(), sqlx::Error> {
    for (name, source) in rules {
        sqlx::query(
            "INSERT INTO yara_rules (rule_name, rule_source)
             VALUES ($1, $2)
             ON CONFLICT (rule_name) DO UPDATE SET
               rule_source = EXCLUDED.rule_source,
               updated_at = now()"
        )
        .bind(name)
        .bind(source)
        .execute(pool)
        .await?;
    }

    Ok(())
}
