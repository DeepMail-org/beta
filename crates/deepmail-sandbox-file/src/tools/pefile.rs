/// pefile — PE header analysis via inline Python script.

use std::path::Path;

use crate::error::SandboxFileError;
use super::run_tool_with_timeout;

const PEFILE_SCRIPT: &str = r#"
import pefile, json, sys
try:
    pe = pefile.PE(sys.argv[1])
    imports = []
    if hasattr(pe, 'DIRECTORY_ENTRY_IMPORT'):
        for entry in pe.DIRECTORY_ENTRY_IMPORT:
            dll = entry.dll.decode('utf-8', errors='replace')
            for imp in entry.imports:
                if imp.name:
                    imports.append({'dll': dll,
                      'func': imp.name.decode('utf-8', errors='replace')})
    result = {
        'is_pe': True,
        'is_dll': pe.is_dll(),
        'is_exe': pe.is_exe(),
        'machine': hex(pe.FILE_HEADER.Machine),
        'timestamp': pe.FILE_HEADER.TimeDateStamp,
        'num_sections': pe.FILE_HEADER.NumberOfSections,
        'imports': imports[:100],
        'is_packed': len(pe.sections) < 3 and
                     pe.FILE_HEADER.NumberOfSections > 0,
        'has_signature': bool(pe.OPTIONAL_HEADER.DATA_DIRECTORY[4].Size)
    }
except Exception as e:
    result = {'is_pe': False, 'error': str(e)}
print(json.dumps(result))
"#;

/// Suspicious PE imports to check against.
const SUSPICIOUS_IMPORTS: &[&str] = &[
    "VirtualAlloc", "VirtualProtect", "WriteProcessMemory", "CreateRemoteThread",
    "LoadLibraryA", "LoadLibraryW", "GetProcAddress", "SetWindowsHookEx",
    "IsDebuggerPresent", "CheckRemoteDebuggerPresent", "NtQueryInformationProcess",
    "CreateToolhelp32Snapshot", "OpenProcess", "TerminateProcess", "RegSetValueEx",
    "URLDownloadToFile", "WinExec", "ShellExecuteA", "CreateProcessA", "SuspendThread",
];

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct PeResult {
    pub is_pe: bool,
    pub is_dll: bool,
    pub is_exe: bool,
    pub machine: String,
    pub num_sections: i32,
    pub imports: Vec<PeImport>,
    pub is_packed: bool,
    pub has_signature: bool,
    pub suspicious_imports: Vec<String>,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct PeImport {
    pub dll: String,
    pub func_name: String,
}

/// Run pefile analysis via inline Python.
pub async fn run_pefile(path: &Path, timeout: u64) -> Result<PeResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    let result = match run_tool_with_timeout(
        "python3", &["-c", PEFILE_SCRIPT, &path_str], None, timeout,
    ).await {
        Ok(r) => r,
        Err(SandboxFileError::ToolNotFound(_)) => {
            tracing::warn!("python3 not available, falling back to MZ check");
            return Ok(detect_pe_by_magic(path));
        }
        Err(e) => return Err(e),
    };

    if result.timed_out {
        return Ok(detect_pe_by_magic(path));
    }

    let json: serde_json::Value = match serde_json::from_str(result.stdout.trim()) {
        Ok(v) => v,
        Err(_) => {
            tracing::warn!("pefile output not JSON, falling back to MZ check");
            return Ok(detect_pe_by_magic(path));
        }
    };

    let is_pe = json.get("is_pe").and_then(|v| v.as_bool()).unwrap_or(false);
    if !is_pe {
        return Ok(PeResult::default());
    }

    let is_dll = json.get("is_dll").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_exe = json.get("is_exe").and_then(|v| v.as_bool()).unwrap_or(false);
    let machine = json.get("machine").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let num_sections = json.get("num_sections").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let is_packed = json.get("is_packed").and_then(|v| v.as_bool()).unwrap_or(false);
    let has_signature = json.get("has_signature").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut imports = Vec::new();
    if let Some(arr) = json.get("imports").and_then(|v| v.as_array()) {
        for item in arr {
            let dll = item.get("dll").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let func = item.get("func").and_then(|v| v.as_str()).unwrap_or("").to_string();
            imports.push(PeImport { dll, func_name: func });
        }
    }

    // Find suspicious imports
    let suspicious_imports: Vec<String> = imports
        .iter()
        .filter(|imp| {
            SUSPICIOUS_IMPORTS.iter().any(|s| imp.func_name.eq_ignore_ascii_case(s))
        })
        .map(|imp| imp.func_name.clone())
        .collect();

    Ok(PeResult {
        is_pe,
        is_dll,
        is_exe,
        machine,
        num_sections,
        imports,
        is_packed,
        has_signature,
        suspicious_imports,
    })
}

/// Detect PE by MZ magic bytes when python3/pefile unavailable.
fn detect_pe_by_magic(path: &Path) -> PeResult {
    if let Ok(data) = std::fs::read(path) {
        if data.len() >= 2 && data[0] == 0x4D && data[1] == 0x5A {
            return PeResult { is_pe: true, ..Default::default() };
        }
    }
    PeResult::default()
}
