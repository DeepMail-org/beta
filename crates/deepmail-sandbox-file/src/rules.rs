/// Embedded YARA rules and compilation.

/// Default YARA rules for static file analysis.
pub const YARA_RULES: &str = r#"
rule SuspiciousMacroKeywords {
  meta:
    description = "Detects common macro-based attack keywords"
  strings:
    $a = "AutoOpen" nocase
    $b = "AutoExec" nocase
    $c = "Shell" nocase
    $d = "WScript.Shell" nocase
    $e = "PowerShell" nocase
    $f = "cmd.exe" nocase
    $g = "CreateObject" nocase
    $h = "GetObject" nocase
    $i = "environ" nocase
    $j = "URLDownloadToFile" nocase
  condition:
    3 of them
}

rule SuspiciousStrings {
  meta:
    description = "Generic suspicious string patterns"
  strings:
    $a = "http://" nocase
    $b = "https://" nocase
    $c = "ftp://" nocase
    $d = /[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/
    $e = "base64" nocase
    $f = "eval(" nocase
    $g = "exec(" nocase
  condition:
    4 of them
}

rule PeExecutable {
  meta:
    description = "Detects Windows PE executable files"
  strings:
    $mz = { 4D 5A }
  condition:
    $mz at 0
}

rule PdfWithJavaScript {
  meta:
    description = "Detects PDF files containing JavaScript"
  strings:
    $pdf = "%PDF"
    $js1 = "/JavaScript" nocase
    $js2 = "/JS" nocase
  condition:
    $pdf at 0 and ($js1 or $js2)
}

rule SuspiciousEmbedded {
  meta:
    description = "Detects files with suspicious embedded content"
  strings:
    $pe = { 4D 5A }
    $elf = { 7F 45 4C 46 }
    $zip = { 50 4B 03 04 }
    $rar = { 52 61 72 21 }
  condition:
    2 of them
}
"#;

/// Compile the embedded YARA rules into a yara_x::Rules object.
pub fn compile_rules() -> Result<yara_x::Rules, String> {
    let mut compiler = yara_x::Compiler::new();
    compiler
        .add_source(YARA_RULES)
        .map_err(|e| format!("YARA compile error: {}", e))?;
    let rules = compiler.build();
    Ok(rules)
}
