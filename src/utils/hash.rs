use sha2::{Digest, Sha256};

/// Hash function used for sensitive data
pub fn hash_str(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    // Encode hexadecimals into string
    hex::encode(result)
}
