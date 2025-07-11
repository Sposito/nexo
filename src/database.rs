use sqlx::Row;

use rocket_db_pools::Database;
use rocket_db_pools::*;

#[derive(Database)]
#[database("nexo_db")]
pub struct NexoDB(rocket_db_pools::sqlx::SqlitePool);

pub async fn get_password_hash_from_username(db: &NexoDB, username: &str) -> Option<String>{
    let result = sqlx::query("SELECT name, psw_hash FROM users WHERE name = ?")
        .bind(username.to_ascii_lowercase())
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