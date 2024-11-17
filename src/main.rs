use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use log::{info, error};
use actix_cors::Cors; // Import CORS

struct AppState {
    conn: Arc<Mutex<Connection>>,
}

/// Initialize the SQLite database and create the counter table.
fn init_db() -> Result<Connection> {
    let conn = Connection::open("counter.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS counter (value INTEGER NOT NULL)",
        [],
    )?;

    // Initialize counter to 0 if the table is empty.
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM counter", [], |row| row.get(0))?;
    if count == 0 {
        conn.execute("INSERT INTO counter (value) VALUES (0)", [])?;
        info!("Counter initialized to 0");
    } else {
        info!("Counter already exists with value: {}", count);
    }

    Ok(conn)
}

/// Increment the counter in a thread-safe manner.
fn increment_counter(conn: &mut Connection) -> Result<i64> {
    let tx = conn.transaction()?;

    // Lock the database row for updates.
    let current_value: i64 = tx.query_row("SELECT value FROM counter", [], |row| row.get(0))?;
    let new_value = current_value + 1;
    tx.execute("UPDATE counter SET value = ?", params![new_value])?;
    tx.commit()?;

    info!("Counter incremented to {}", new_value);
    Ok(new_value)
}

/// API handler to increment and return the counter value.
async fn visit(data: web::Data<AppState>) -> impl Responder {
    let mut conn = data.conn.lock().unwrap();

    match increment_counter(&mut conn) {
        Ok(value) => {
            info!("Counter updated successfully, new value: {}", value);
            HttpResponse::Ok().json(value)
        }
        Err(err) => {
            error!("Error incrementing counter: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init();

    info!("Starting the server...");

    let conn = init_db().expect("Failed to initialize database");
    let data = web::Data::new(AppState {
        conn: Arc::new(Mutex::new(conn)),
    });

    HttpServer::new(move || {
        let conn = init_db().expect("Failed to initialize database");
        let data = web::Data::new(AppState {
            conn: Arc::new(Mutex::new(conn)),
        });
        App::new()
            .app_data(data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin() // Allow any origin (modify as needed for production)
                    .allowed_methods(vec!["POST"]) // Allow specific methods
                    .allowed_headers(vec!["Content-Type"]), // Allow specific headers
            )
            .route("/visit/", web::post().to(visit))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
