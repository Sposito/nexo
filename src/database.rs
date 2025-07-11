use crate::crypto;
use sqlx::Row;

use rocket_db_pools::Database;
use rocket_db_pools::*;

#[derive(Database)]
#[database("nexo_db")]
pub struct Nexodb(rocket_db_pools::sqlx::SqlitePool);


pub async fn validate_user(db: &Nexodb, username: &str, password: &str) -> bool {
    let result = sqlx::query("SELECT name, psw_hash FROM users WHERE name = ?")
        .bind(username.to_ascii_lowercase())
        .fetch_one(&db.0)
        .await;
    
    match result {
        Ok(row) => {
            let hashed_password: String = crypto::hash_password("salt", password);
            let psw_hash: String = row.get("psw_hash");
            println!("Found user: {}, stored hash: {}, computed hash: {}", 
                    username, psw_hash, hashed_password);
            psw_hash == hashed_password
        }
        Err(e) => {
            println!("Database error: {:?}", e);
            false
        }
    }
}