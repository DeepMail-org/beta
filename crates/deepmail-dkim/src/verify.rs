use base64::Engine;
use sha2::{Digest, Sha256};

use crate::error::DkimError;

pub struct VerificationResult {
    pub valid: bool,
    pub algorithm: String,
    pub error: Option<String>,
}

pub fn compute_body_hash(canonical_body: &[u8]) -> String {
    let hash = Sha256::digest(canonical_body);
    base64::engine::general_purpose::STANDARD.encode(hash)
}

pub fn verify_signature(
    algorithm: &str,
    canonical_headers: &[u8],
    signature_b64: &str,
    public_key_data: &str,
    key_algorithm: Option<&str>,
) -> VerificationResult {
    let sig_bytes = match base64::engine::general_purpose::STANDARD.decode(signature_b64) {
        Ok(b) => b,
        Err(e) => {
            return VerificationResult {
                valid: false,
                algorithm: algorithm.to_string(),
                error: Some(format!("invalid base64 signature: {e}")),
            };
        }
    };

    let detected_algo = key_algorithm.unwrap_or(
        if algorithm.contains("ed25519") { "ed25519" } else { "rsa" }
    );

    match detected_algo {
        "ed25519" => verify_ed25519(canonical_headers, &sig_bytes, public_key_data, algorithm),
        _ => verify_rsa_sha256(canonical_headers, &sig_bytes, public_key_data, algorithm),
    }
}

fn verify_rsa_sha256(
    data: &[u8],
    signature: &[u8],
    public_key_b64: &str,
    algorithm: &str,
) -> VerificationResult {
    use rsa::pkcs8::DecodePublicKey;
    use rsa::{Pkcs1v15Sign, RsaPublicKey};

    let key_bytes = match base64::engine::general_purpose::STANDARD.decode(public_key_b64) {
        Ok(b) => b,
        Err(e) => {
            return VerificationResult {
                valid: false,
                algorithm: algorithm.to_string(),
                error: Some(format!("invalid base64 public key: {e}")),
            };
        }
    };

    let public_key = match RsaPublicKey::from_public_key_der(&key_bytes) {
        Ok(k) => k,
        Err(e) => {
            return VerificationResult {
                valid: false,
                algorithm: algorithm.to_string(),
                error: Some(format!("invalid RSA public key DER: {e}")),
            };
        }
    };

    let hash = Sha256::digest(data);
    let scheme = Pkcs1v15Sign::new::<Sha256>();

    match public_key.verify(scheme, &hash, signature) {
        Ok(()) => VerificationResult {
            valid: true,
            algorithm: algorithm.to_string(),
            error: None,
        },
        Err(e) => VerificationResult {
            valid: false,
            algorithm: algorithm.to_string(),
            error: Some(format!("RSA verification failed: {e}")),
        },
    }
}

fn verify_ed25519(
    data: &[u8],
    signature: &[u8],
    public_key_b64: &str,
    algorithm: &str,
) -> VerificationResult {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let key_bytes = match base64::engine::general_purpose::STANDARD.decode(public_key_b64) {
        Ok(b) => b,
        Err(e) => {
            return VerificationResult {
                valid: false,
                algorithm: algorithm.to_string(),
                error: Some(format!("invalid base64 public key: {e}")),
            };
        }
    };

    let key_array: [u8; 32] = match key_bytes.as_slice().try_into() {
        Ok(a) => a,
        Err(_) => {
            return VerificationResult {
                valid: false,
                algorithm: algorithm.to_string(),
                error: Some(format!(
                    "ed25519 public key must be 32 bytes, got {}",
                    key_bytes.len()
                )),
            };
        }
    };

    let verifying_key = match VerifyingKey::from_bytes(&key_array) {
        Ok(k) => k,
        Err(e) => {
            return VerificationResult {
                valid: false,
                algorithm: algorithm.to_string(),
                error: Some(format!("invalid ed25519 key: {e}")),
            };
        }
    };

    let sig = match Signature::from_slice(signature) {
        Ok(s) => s,
        Err(e) => {
            return VerificationResult {
                valid: false,
                algorithm: algorithm.to_string(),
                error: Some(format!("invalid ed25519 signature bytes: {e}")),
            };
        }
    };

    match verifying_key.verify(data, &sig) {
        Ok(()) => VerificationResult {
            valid: true,
            algorithm: algorithm.to_string(),
            error: None,
        },
        Err(e) => VerificationResult {
            valid: false,
            algorithm: algorithm.to_string(),
            error: Some(format!("ed25519 verification failed: {e}")),
        },
    }
}

pub fn parse_dns_key_record(txt: &str) -> Result<DnsPublicKey, DkimError> {
    let mut p_value = None;
    let mut k_value = None;

    for part in txt.split(';') {
        let trimmed = part.trim();
        if let Some(eq_pos) = trimmed.find('=') {
            let tag = trimmed[..eq_pos].trim();
            let val = trimmed[eq_pos + 1..].trim();
            match tag {
                "p" => p_value = Some(val.replace(|c: char| c.is_whitespace(), "")),
                "k" => k_value = Some(val.to_lowercase()),
                _ => {}
            }
        }
    }

    let public_key = p_value
        .ok_or_else(|| DkimError::Parse("DNS key record missing p= tag".to_string()))?;

    if public_key.is_empty() {
        return Err(DkimError::Crypto("key has been revoked (empty p= tag)".to_string()));
    }

    let algorithm = k_value.unwrap_or_else(|| "rsa".to_string());

    Ok(DnsPublicKey {
        public_key_b64: public_key,
        algorithm,
    })
}

pub struct DnsPublicKey {
    pub public_key_b64: String,
    pub algorithm: String,
}
