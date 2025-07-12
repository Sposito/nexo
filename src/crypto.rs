use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::RngCore;

/// A pure function that takes a salt and password and returns a hash.
/// 
/// This function uses SHA-256 to create a secure hash of the password
/// combined with the salt. The salt is prepended to the password before hashing.
/// 
/// # Arguments
/// * `salt` - A string used to salt the password
/// * `password` - The password to hash
/// 
/// # Returns
/// A hexadecimal string representation of the hash
pub fn hash_password(salt: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    Digest::update(&mut hasher, salt.as_bytes());
    Digest::update(&mut hasher, password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Generate a secure random token for session management
/// 
/// This function creates a cryptographically secure token by combining
/// the current timestamp with cryptographically secure random data.
/// This ensures uniqueness even in high-throughput or multithreaded environments.
/// 
/// # Returns
/// A hexadecimal string representation of the token
pub fn generate_session_token() -> String {
    let mut hasher = Sha256::new();
    
    // Get current timestamp for additional entropy
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    // Generate cryptographically secure random bytes
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 32]; // 256 bits of randomness
    rng.fill_bytes(&mut random_bytes);
    
    // Combine timestamp and random bytes for maximum entropy
    let entropy = format!("{}{}", timestamp, std::process::id());
    
    Digest::update(&mut hasher, entropy.as_bytes());
    Digest::update(&mut hasher, &random_bytes);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Get current timestamp as Unix timestamp
pub fn get_current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let salt = "salt";
        let password = "1234";
        let hash1 = hash_password(salt, password);
        let hash2 = hash_password(salt, password);
        
        // Same inputs should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different salt should produce different hash
        let hash3 = hash_password("differentsalt", password);
        assert_ne!(hash1, hash3);
        
        // Different password should produce different hash
        let hash4 = hash_password(salt, "differentpassword");
        assert_ne!(hash1, hash4);
    }

    #[test]
    fn test_empty_password() {
        let hash = hash_password("salt", "");
        assert!(!hash.is_empty());
    }
    
    #[test]
    fn test_know_hash(){
        let salt = "salt";
        let password = "1234";
        let hash1 = hash_password(salt, password);

        // This hash was generated using: echo -n "salt1234" | sha256sum
        let password_hash = "ea32961dbd579ef5697c367f9267921ee07f14d77fb2d4fb9500d4221d615695";
        assert_eq!(hash1.as_str(), password_hash);
    }
    #[test]
    fn test_empty_salt() {
        let hash = hash_password("", "password");
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_hash_output_is_hex() {
        let hash = hash_password("salt", "password");
        
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_session_token() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();
        
        // Tokens should be different (due to timestamp)
        assert_ne!(token1, token2);
        
        // Tokens should be 64 characters (SHA-256 hex)
        assert_eq!(token1.len(), 64);
        assert_eq!(token2.len(), 64);
        
        // Tokens should be valid hex
        assert!(token1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(token2.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_token_uniqueness_under_load() {
        use std::collections::HashSet;
        
        // Generate many tokens rapidly to test uniqueness
        let mut tokens = HashSet::new();
        let mut duplicates = 0;
        
        for _ in 0..1000 {
            let token = generate_session_token();
            if !tokens.insert(token) {
                duplicates += 1;
            }
        }
        
        // Should have no duplicates
        assert_eq!(duplicates, 0, "Found {} duplicate tokens", duplicates);
        assert_eq!(tokens.len(), 1000, "Expected 1000 unique tokens, got {}", tokens.len());
    }

    #[test]
    fn test_token_uniqueness_multithreaded() {
        use std::collections::HashSet;
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let tokens = Arc::new(Mutex::new(HashSet::new()));
        let mut handles = vec![];
        
        // Spawn multiple threads to generate tokens concurrently
        for _ in 0..10 {
            let tokens_clone = Arc::clone(&tokens);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let token = generate_session_token();
                    tokens_clone.lock().unwrap().insert(token);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Check that all tokens are unique
        let final_tokens = tokens.lock().unwrap();
        assert_eq!(final_tokens.len(), 1000, "Expected 1000 unique tokens, got {}", final_tokens.len());
    }

    #[test]
    fn test_get_current_timestamp() {
        let timestamp1 = get_current_timestamp();
        let timestamp2 = get_current_timestamp();
        
        // Timestamps should be reasonable (not too far in past/future)
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        assert!(timestamp1 > 0);
        assert!(timestamp2 > 0);
        assert!(timestamp1 <= now + 1); // Allow 1 second difference
        assert!(timestamp2 <= now + 1);
    }
}
