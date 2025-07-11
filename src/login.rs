use rocket::response::content::RawHtml;
use rocket::form::Form;
use crate::crypto::{hash_password};
use crate::database::{NexoDB, get_password_hash_from_username as get_psw};

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/", data = "<form>")]
pub async fn login(form: Form<LoginForm>, db: &NexoDB) -> RawHtml<String> {
    let stored_hash = get_user_psw_from_db(db, form.username.clone()).await;
    
    let is_valid = validate_user_psw(stored_hash, form.password.clone(), "salt" );
    if is_valid {
        println!("Login successful");
        RawHtml(r#"<div class="alert alert-success">Login successful!</div>"#.to_string())
    } else {
        println!("Invalid username or password");
        RawHtml(r#"<div class="alert alert-danger">Invalid username or password</div>"#.to_string())
    }
}

async fn get_user_psw_from_db(db: &NexoDB, username: String) -> Option<String>{
    let result = get_psw(db, username.as_str()).await;
    result
}
fn validate_user_psw(stored_password_hash: Option<String>, login_password: String, salt:&str) -> bool {
    match stored_password_hash {
        Some(from_db) => {
            from_db == hash_password(salt, login_password.as_str())
        }
        None => false
    }
}

#[cfg(test)]
mod tests {
    use super::validate_user_psw;
    use crate::crypto::hash_password;

    const SALT: &str = "test_salt";

    #[test]
    fn test_valid_password() {
        let password = "secure_password";
        let hashed = hash_password(SALT, password);
        let result = validate_user_psw(Some(hashed), password.to_string(), SALT);
        assert!(result, "Password should be valid");
    }

    #[test]
    fn test_invalid_password() {
        let correct_password = "secure_password";
        let wrong_password = "wrong_password";
        let hashed = hash_password(SALT, correct_password);
        let result = validate_user_psw(Some(hashed), wrong_password.to_string(), SALT);
        assert!(!result, "Password should be invalid");
    }

    #[test]
    fn test_none_stored_hash() {
        let result = validate_user_psw(None, "any_password".to_string(), SALT);
        assert!(!result, "Validation should fail if no stored hash is found");
    }

    #[test]
    fn test_empty_password() {
        let hashed = hash_password(SALT, "");
        let result = validate_user_psw(Some(hashed), "".to_string(), SALT);
        assert!(result, "Empty password should match its correct hash");
    }

    #[test]
    fn test_mismatched_hash() {
        let password = "password";
        let hashed = hash_password(SALT, password);
        let tampered_hash = format!("{}_modified", hashed);
        let result = validate_user_psw(Some(tampered_hash), password.to_string(), SALT);
        assert!(!result, "Hash mismatch should result in invalidation");
    }

    #[test]
    fn test_password_with_special_chars() {
        let password = "!@#$%^&*()_+-=[]{}|;':,.<>?";
        let hashed = hash_password(SALT, password);
        let result = validate_user_psw(Some(hashed), password.to_string(), SALT);
        assert!(result, "Password with special characters should be valid");
    }
}

