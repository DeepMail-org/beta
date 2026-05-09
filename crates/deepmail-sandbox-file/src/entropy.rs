/// Shannon entropy calculation for file analysis.

/// Compute Shannon entropy of a byte slice.
///
/// Returns a value in the range [0.0, 8.0].
/// - 0.0 = perfectly uniform (single byte repeated)
/// - ~7.0 = compressed data
/// - > 7.5 = likely encrypted or packed
pub fn compute_entropy(data: &[u8]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }

    let mut freq = [0u64; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0f64;

    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy as f32
}

/// Classify entropy value into a human-readable verdict.
#[allow(dead_code)]
pub fn entropy_verdict(entropy: f32) -> &'static str {
    if entropy > 7.5 {
        "likely_encrypted_or_packed"
    } else if entropy > 7.0 {
        "high_entropy_suspicious"
    } else if entropy > 6.0 {
        "medium_entropy_compressed"
    } else {
        "normal"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_slice() {
        assert_eq!(compute_entropy(&[]), 0.0);
    }

    #[test]
    fn test_all_same_byte() {
        let data = vec![0x41u8; 1000];
        assert_eq!(compute_entropy(&data), 0.0);
    }

    #[test]
    fn test_random_like_data() {
        // Create data with all 256 byte values equally distributed
        let mut data = Vec::with_capacity(256 * 100);
        for _ in 0..100 {
            for b in 0u8..=255 {
                data.push(b);
            }
        }
        let entropy = compute_entropy(&data);
        // Perfectly uniform distribution of 256 symbols → entropy = 8.0
        assert!(entropy > 7.9, "entropy should be near 8.0, got {}", entropy);
    }

    #[test]
    fn test_entropy_verdict() {
        assert_eq!(entropy_verdict(7.8), "likely_encrypted_or_packed");
        assert_eq!(entropy_verdict(7.2), "high_entropy_suspicious");
        assert_eq!(entropy_verdict(6.5), "medium_entropy_compressed");
        assert_eq!(entropy_verdict(4.0), "normal");
    }
}
