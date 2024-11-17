use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};

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
    Ok(new_value)
}


/// API handler to increment and return the counter value.
async fn visit(data: web::Data<AppState>) -> impl Responder {
    let mut conn = data.conn.lock().unwrap();

    match increment_counter(&mut conn) {
        Ok(value) => HttpResponse::Ok().json(value),
        Err(err) => {
            eprintln!("Error incrementing counter: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let conn = init_db().expect("Failed to initialize database");
    let data = web::Data::new(AppState {
        conn: Arc::new(Mutex::new(conn)),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/visit", web::post().to(visit))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
