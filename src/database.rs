use sqlx::Row;

use rocket_db_pools::Database;
use rocket_db_pools::*;

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


}
