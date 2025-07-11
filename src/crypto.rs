use sha2::{Sha256, Digest};

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

}
