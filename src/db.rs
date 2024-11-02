// db.rs
use rusqlite::{params, Connection, Result};

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create visitors table with unique IP addresses
        conn.execute(
            "CREATE TABLE IF NOT EXISTS visitors (
                ip TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        // Create visitor_count table with one row to store the count of unique visitors
        conn.execute(
            "CREATE TABLE IF NOT EXISTS visitor_count (
                count INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Initialize visitor_count if it's empty
        if conn.query_row("SELECT count FROM visitor_count", [], |row| row.get::<_, i32>(0)).is_err() {
            conn.execute("INSERT INTO visitor_count (count) VALUES (0)", [])?;
        }

        Ok(Db { conn })
    }

    // Method to handle unique visitors
    pub fn visit(&self, ip: &str) -> Result<usize> {
        // Attempt to insert the new IP address using INSERT OR IGNORE
        let inserted = self.conn.execute("INSERT OR IGNORE INTO visitors (ip) VALUES (?)", params![ip])?;

        // If a new IP was inserted, increment the unique visitor count
        if inserted > 0 {
            self.increment_unique_count()?;
        }

        // Return the current unique visitor count
        self.unique_count()
    }

    // Increment the unique visitor count
    fn increment_unique_count(&self) -> Result<()> {
        self.conn.execute("UPDATE visitor_count SET count = count + 1", [])?;
        Ok(())
    }

    // Get the unique visitor count
    pub fn unique_count(&self) -> Result<usize> {
        let count: i32 = self.conn.query_row(
            "SELECT count FROM visitor_count",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize) // Convert to usize for return type
    }
}
