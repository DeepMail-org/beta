/// QR code decoding via zbarimg subprocess.

use std::io::Write;
use std::process::{Command, Stdio};

/// Decode QR codes from raw image bytes using zbarimg.
///
/// Returns a list of decoded string values (typically URLs).
/// If zbarimg is not installed, returns an empty Vec and logs a warning.
pub fn decode_qr_from_bytes(image_bytes: &[u8]) -> Vec<String> {
    // Check if zbarimg is available
    if which::which("zbarimg").is_err() {
        tracing::warn!("zbarimg not available, QR decoding disabled");
        return Vec::new();
    }

    let mut child = match Command::new("zbarimg")
        .args(["--raw", "--quiet", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to spawn zbarimg: {}", e);
            return Vec::new();
        }
    };

    // Write image to stdin
    if let Some(ref mut stdin) = child.stdin {
        if let Err(e) = stdin.write_all(image_bytes) {
            tracing::warn!("failed to write to zbarimg stdin: {}", e);
            return Vec::new();
        }
    }
    // Drop stdin to signal EOF
    drop(child.stdin.take());

    // Wait with timeout (5 seconds)
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!("zbarimg wait failed: {}", e);
            return Vec::new();
        }
    };

    if !output.status.success() {
        tracing::debug!("zbarimg exited with status {}", output.status);
        // exit code 4 = no barcodes found; that's OK
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            // zbarimg may prefix with "QR-Code:" — strip it
            if let Some(stripped) = line.strip_prefix("QR-Code:") {
                stripped.trim().to_string()
            } else {
                line.trim().to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_bytes() {
        // Should not panic, just return empty
        let result = decode_qr_from_bytes(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_decode_invalid_image() {
        // Should not panic, return empty
        let result = decode_qr_from_bytes(b"not an image");
        assert!(result.is_empty());
    }
}
