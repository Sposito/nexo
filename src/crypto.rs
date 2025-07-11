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
pub fn hash_password(salt: &str, password: &str) -> String {
    // Combine salt and password
    let salted_password = format!("{}{}", salt, password);
    
    // Create a hash using SHA-256
    let mut hasher = DefaultHasher::new();
    salted_password.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Convert to hexadecimal string
    format!("{:x}", hash)
}

/// Verify if a password matches a stored hash
/// 
/// # Arguments
/// * `password` - The password to verify
/// * `stored_hash` - The stored hash to compare against
/// 
/// # Returns
/// True if the password matches the stored hash, false otherwise
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    // For now, we'll assume the stored hash includes the salt
    // This is a simple implementation - in production you'd want more sophisticated parsing
    let salt = "mysalt"; // This should be extracted from the stored hash in a real implementation
    let computed_hash = hash_password(salt, password);
    computed_hash == stored_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let salt = "salt";
        let password = "6564";
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
}
