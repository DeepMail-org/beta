/// YARA scanning using yara-x (pure Rust).

use crate::error::SandboxFileError;

/// Run YARA scan against file data. Returns matched rule names.
pub async fn run_yara_scan(
    rules: &yara_x::Rules,
    data: &[u8],
) -> Result<Vec<String>, SandboxFileError> {
    // Clone data for spawn_blocking
    let data = data.to_vec();

    // yara-x Rules is not Send, so we need to serialize/deserialize
    // Actually yara-x::Rules implements Send+Sync in v1.x, but scanner needs &Rules
    // We'll use a direct approach since Rules is behind Arc in pipeline

    let mut scanner = yara_x::Scanner::new(rules);
    let scan_results = scanner
        .scan(&data)
        .map_err(|e| SandboxFileError::Yara(format!("scan error: {}", e)))?;

    let matched: Vec<String> = scan_results
        .matching_rules()
        .map(|rule| rule.identifier().to_string())
        .collect();

    Ok(matched)
}
