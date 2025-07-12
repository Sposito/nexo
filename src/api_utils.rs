use rocket::serde::json::Json;
use rocket::http::Status;
use crate::database::{NexoDB, ensure_db_initialized};
use serde_json::json;

#[post("/init-db")]
pub async fn init_db_endpoint(db: &NexoDB) -> (Status, Json<serde_json::Value>) {
    match ensure_db_initialized(db).await {
        Ok(_) => (Status::Ok, Json(json!({"status": "ok", "message": "Database initialized"}))),
        Err(e) => (Status::InternalServerError, Json(json!({"status": "error", "error": format!("{}", e)}))),
    }
}
