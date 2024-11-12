use actix_web::{web, App, HttpServer, HttpResponse, Responder, HttpRequest};
use std::sync::Arc;
use tokio::sync::Mutex;
use actix_cors::Cors; // Import CORS
use log::{info, error}; // Import logging macros
use env_logger; // Import env_logger for logging
use std::fs;
use serde_json::json; // Import for JSON response
mod db;
use db::Db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init();

    // Ensure the "database" directory exists
    fs::create_dir_all("database").expect("Failed to create database directory");

    // Specify the path to the database file within the "database" folder
    let db_path = "database/visitors.db";

    // Get the full path of the database file
    let full_db_path = std::env::current_dir()
        .expect("Failed to get current directory")
        .join(db_path);

    // Log the full path
    info!("Database file path: {:?}", full_db_path.display());

    let db = Arc::new(Mutex::new(Db::new(db_path).unwrap()));

    HttpServer::new(move || {
        let db_clone = db.clone();
        App::new()
            .app_data(web::Data::new(db_clone))
            .wrap(
                Cors::default()
                    .allow_any_origin() // Allow any origin (modify as needed for production)
                    .allowed_methods(vec!["POST"]) // Allow specific methods
                    .allowed_headers(vec!["Content-Type"]), // Allow specific headers
            )
            .route("/visit/", web::post().to(record_visit))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

fn get_client_ip(req: &HttpRequest) -> String {
    // Check for the X-Forwarded-For header
    if let Some(ip) = req.headers().get("X-Forwarded-For") {
        if let Ok(ip_str) = ip.to_str() {
            // Take the first IP in the list, which is the real client IP
            return ip_str.split(',').next().unwrap_or("Unknown IP").trim().to_string();
        }
    }

    // Fallback to peer address if header is not present
    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "Unknown IP".to_string())
}

async fn record_visit(db: web::Data<Arc<Mutex<Db>>>, req: HttpRequest) -> impl Responder {
    // Retrieve the client's IP address
    let ip = get_client_ip(&req);

    info!("Received visit request from IP: {}", ip); // Log the incoming request
    let db = db.lock().await;

    match db.visit(&ip) {
        Ok(count) => {
            info!("Successfully recorded visit for IP: {}. Total unique visits: {}", ip, count);
            // Create a JSON response
            let response = json!( {
                "success": true,
                "count": count
            });
            HttpResponse::Ok().json(response) // Return JSON response
        },
        Err(err) => {
            error!("Failed to process visit for IP: {}. Error: {}", ip, err); // Log the error
            let response = json!( {
                "success": false,
                "count": 0 // Count will be zero in case of an error
            });
            HttpResponse::InternalServerError().json(response) // Return JSON response with error status
        },
    }
}

