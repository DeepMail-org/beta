/// exiftool — metadata extraction.

use std::path::Path;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::SandboxFileError;
use super::run_tool_with_timeout;

#[derive(Debug, Clone, Default)]
pub struct ExifResult {
    pub author: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub software: Option<String>,
    pub raw: serde_json::Value,
}

/// Run exiftool and parse JSON output.
pub async fn run_exiftool(path: &Path, timeout: u64) -> Result<ExifResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    let result = match run_tool_with_timeout(
        "exiftool", &["-j", "-q", &path_str], None, timeout,
    ).await {
        Ok(r) => r,
        Err(SandboxFileError::ToolNotFound(_)) => {
            tracing::warn!("exiftool not available, skipping");
            return Ok(ExifResult::default());
        }
        Err(e) => return Err(e),
    };

    if result.timed_out || result.stdout.is_empty() {
        return Ok(ExifResult::default());
    }

    // Parse JSON array
    let parsed: serde_json::Value = match serde_json::from_str(&result.stdout) {
        Ok(v) => v,
        Err(_) => return Ok(ExifResult::default()),
    };

    let obj = match parsed.as_array().and_then(|arr| arr.first()) {
        Some(o) => o.clone(),
        None => return Ok(ExifResult { raw: parsed, ..Default::default() }),
    };

    let author = find_field(&obj, &["Author", "Creator"]);
    let software = find_field(&obj, &["Software"]);

    let created = find_field(&obj, &["CreateDate", "DateTimeOriginal", "CreationDate"])
        .and_then(|s| parse_exif_date(&s));
    let modified = find_field(&obj, &["ModifyDate", "FileModifyDate"])
        .and_then(|s| parse_exif_date(&s));

    Ok(ExifResult {
        author,
        created,
        modified,
        software,
        raw: obj,
    })
}

/// Case-insensitive field lookup from JSON object.
fn find_field(obj: &serde_json::Value, keys: &[&str]) -> Option<String> {
    if let Some(map) = obj.as_object() {
        for key in keys {
            for (k, v) in map {
                if k.eq_ignore_ascii_case(key) {
                    if let Some(s) = v.as_str() {
                        let trimmed = s.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Parse exiftool date strings.
fn parse_exif_date(s: &str) -> Option<DateTime<Utc>> {
    // Try "YYYY:MM:DD HH:MM:SS"
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
    }
    // Try ISO 8601
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try "YYYY:MM:DD HH:MM:SSz"
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S%z") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
    }
    None
}
