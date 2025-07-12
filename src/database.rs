use sqlx::Row;

use rocket_db_pools::Database;
use rocket_db_pools::*;
use crate::crypto::{generate_session_token, get_current_timestamp};

#[derive(Database)]
#[database("nexo_db")]
pub struct NexoDB(rocket_db_pools::sqlx::SqlitePool);

pub async fn init_db(db: &NexoDB) -> Result<(), sqlx::Error> {
    let sql = include_str!("../data/db.sql");
    sqlx::query(sql).execute(&db.0).await?;
    Ok(())
}

// Lazy initialization - call this when you first need the database
pub async fn ensure_db_initialized(db: &NexoDB) -> Result<(), sqlx::Error> {
    // Check if tables exist by trying to query them
    let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
        .fetch_optional(&db.0)
        .await?;
    
    if result.is_none() {
        println!("Database not initialized, running init_db...");
        init_db(db).await?;
        println!("Database initialized successfully");
    }
    
    Ok(())
}

pub async fn get_password_hash_from_username(db: &NexoDB, username: &str) -> Option<String>{
    let sql = "SELECT name, psw_hash FROM users WHERE name = ?";
    let result = sqlx::query(sql)
        .bind(username.to_string())
        .fetch_one(&db.0)
        .await;
    match result {
        Ok(row) =>{
            row.get("psw_hash")
        }
        Err(e) => {
            println!("Database error: {:?}", e);
            None
        }
    }
}

/// Get user ID by username
pub async fn get_user_id_by_username(db: &NexoDB, username: &str) -> Option<i32> {
    let sql = "SELECT id FROM users WHERE name = ?";
    let result = sqlx::query(sql)
        .bind(username.to_string())
        .fetch_one(&db.0)
        .await;
    match result {
        Ok(row) => Some(row.get("id")),
        Err(_) => None,
    }
}

/// Create a new session for a user
pub async fn create_session(db: &NexoDB, user_id: i32, expires_in_seconds: i64) -> Option<String> {
    let token = generate_session_token();
    let expires_at = get_current_timestamp() + expires_in_seconds;
    
    let sql = "INSERT INTO sessions (user_id, token, expires_at) VALUES (?, ?, ?)";
    let result = sqlx::query(sql)
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&db.0)
        .await;
    
    match result {
        Ok(_) => Some(token),
        Err(e) => {
            println!("Failed to create session: {:?}", e);
            None
        }
    }
}

/// Validate a session token and return user ID if valid
pub async fn validate_session(db: &NexoDB, token: &str) -> Option<i32> {
    let current_time = get_current_timestamp();
    
    let sql = "SELECT user_id FROM sessions WHERE token = ? AND expires_at > ?";
    let result = sqlx::query(sql)
        .bind(token)
        .bind(current_time)
        .fetch_one(&db.0)
        .await;
    
    match result {
        Ok(row) => Some(row.get("user_id")),
        Err(_) => None,
    }
}

/// Get username by user ID
pub async fn get_username_by_id(db: &NexoDB, user_id: i32) -> Option<String> {
    let sql = "SELECT name FROM users WHERE id = ?";
    let result = sqlx::query(sql)
        .bind(user_id)
        .fetch_one(&db.0)
        .await;
    
    match result {
        Ok(row) => Some(row.get("name")),
        Err(_) => None,
    }
}

/// Clean up expired sessions
pub async fn cleanup_expired_sessions(db: &NexoDB) -> Result<u64, sqlx::Error> {
    let current_time = get_current_timestamp();
    
    let sql = "DELETE FROM sessions WHERE expires_at <= ?";
    let result = sqlx::query(sql)
        .bind(current_time)
        .execute(&db.0)
        .await?;
    
    Ok(result.rows_affected())
}

/// Delete a specific session by token
pub async fn delete_session(db: &NexoDB, token: &str) -> Result<u64, sqlx::Error> {
    let sql = "DELETE FROM sessions WHERE token = ?";
    let result = sqlx::query(sql)
        .bind(token)
        .execute(&db.0)
        .await?;
    
    Ok(result.rows_affected())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_init_db() {
        rocket::async_test(async {
            let db_path = "test_db.sqlite";

            // Delete the file if it exists
            if Path::new(db_path).exists() {
                std::fs::remove_file(db_path).expect("Failed to delete existing database file");
                println!("Deleted existing database file");
            }

            // Explicitly create an empty file
            match std::fs::File::create(db_path) {
                Ok(_) => println!("Created empty test_db.sqlite file"),
                Err(e) => println!("Failed to create test_db.sqlite file: {}", e),
            }

            // Print file metadata
            match std::fs::metadata(db_path) {
                Ok(meta) => println!("test_db.sqlite metadata: permissions: {:?}, len: {}", meta.permissions(), meta.len()),
                Err(e) => println!("Failed to get metadata: {}", e),
            }

            // Create a test database connection
            let database_url = format!("sqlite://{}", db_path);
            let pool = sqlx::SqlitePool::connect(&database_url)
                .await
                .expect("Failed to create database pool");
            let db = NexoDB(pool);

            // Initialize the database
            println!("Initializing database...");
            init_db(&db).await.expect("Failed to initialize database");
            println!("Database initialized successfully");

            // Test if it was correctly initialized by running a query
            println!("Testing database initialization...");
            let result = sqlx::query("SELECT COUNT(*) as count FROM users")
                .fetch_one(&db.0)
                .await
                .expect("Failed to query users table");
            
            let user_count: i64 = result.get("count");
            println!("Found {} users in the database", user_count);
            
            // Test the specific user that should be inserted
            let username = "thiago";
            let password_hash = get_password_hash_from_username(&db, username).await;
            match password_hash {
                Some(hash) => println!("Found user '{}' with hash: {}", username, hash),
                None => println!("User '{}' not found", username),
            }

            // Test sessions table
            let sessions_result = sqlx::query("SELECT COUNT(*) as count FROM sessions")
                .fetch_one(&db.0)
                .await
                .expect("Failed to query sessions table");
            
            let sessions_count: i64 = sessions_result.get("count");
            println!("Found {} sessions in the database", sessions_count);

            // Close connection
            db.0.close().await;
            println!("Database connection closed");

            // Delete the file
            if Path::new(db_path).exists() {
                fs::remove_file(db_path).expect("Failed to delete test database file");
                println!("Test database file deleted");
            }
            
            println!("Test completed successfully!");
        });
    }

    #[test]
    fn test_session_management() {
        rocket::async_test(async {
            let db_path = "test_session_db.sqlite";

            // Clean up existing file
            if Path::new(db_path).exists() {
                std::fs::remove_file(db_path).expect("Failed to delete existing database file");
            }

            // Create an empty file first
            std::fs::File::create(db_path).expect("Failed to create database file");

            // Create database
            let database_url = format!("sqlite://{}", db_path);
            let pool = sqlx::SqlitePool::connect(&database_url)
                .await
                .expect("Failed to create database pool");
            let db = NexoDB(pool);

            // Initialize database
            init_db(&db).await.expect("Failed to initialize database");

            // Test user lookup
            let user_id = get_user_id_by_username(&db, "thiago").await;
            assert!(user_id.is_some());
            let user_id = user_id.unwrap();

            // Test session creation
            let session_token = create_session(&db, user_id, 3600).await; // 1 hour
            assert!(session_token.is_some());
            let session_token = session_token.unwrap();

            // Test session validation
            let validated_user_id = validate_session(&db, &session_token).await;
            assert_eq!(validated_user_id, Some(user_id));

            // Test username lookup
            let username = get_username_by_id(&db, user_id).await;
            assert_eq!(username, Some("thiago".to_string()));

            // Test session deletion
            let deleted_count = delete_session(&db, &session_token).await;
            assert_eq!(deleted_count.unwrap(), 1);

            // Test session validation after deletion
            let validated_user_id = validate_session(&db, &session_token).await;
            assert!(validated_user_id.is_none());

            // Test expired session
            let expired_token = create_session(&db, user_id, -1).await; // Expired immediately
            assert!(expired_token.is_some());
            let expired_token = expired_token.unwrap();

            let validated_user_id = validate_session(&db, &expired_token).await;
            assert!(validated_user_id.is_none());

            // Clean up
            db.0.close().await;
            if Path::new(db_path).exists() {
                fs::remove_file(db_path).expect("Failed to delete test database file");
            }
        });
    }
}
