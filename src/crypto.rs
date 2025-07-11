use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
pub fn hash_password(salt: &str, password: String) -> String {
    // Combine salt and password
    let salted_password = format!("{}{}", salt, password);
    
    // Create a hash using SHA-256
    let mut hasher = DefaultHasher::new();
    salted_password.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Convert to hexadecimal string
    format!("{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let salt = "salt";
        let password = "6564";
        let hash1 = hash_password(salt, password.to_string());
        
        let hash2 = hash_password(salt, password.to_string());
        
        // Same inputs should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different salt should produce different hash
        let hash3 = hash_password("differentsalt", password.to_string());
        assert_ne!(hash1, hash3);
        
        // Different password should produce different hash
        let hash4 = hash_password(salt, "differentpassword".to_string());
        assert_ne!(hash1, hash4);
    }
}
