mod confession;
mod reply;
mod vote;

use rusqlite::{Connection, Result as SqlResult};

pub use confession::{get_all, insert, posts_today, stats};
pub use reply::{get_replies, insert_reply};
pub use vote::{upvote, voted_confession_ids};

pub fn init(path: &str) -> SqlResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS confessions (
            id INTEGER PRIMARY KEY,
            text TEXT NOT NULL,
            x INTEGER NOT NULL,
            y INTEGER NOT NULL,
            votes INTEGER DEFAULT 0,
            author_fingerprint TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS votes (
            confession_id INTEGER,
            voter_fingerprint TEXT,
            PRIMARY KEY (confession_id, voter_fingerprint)
        );
        CREATE TABLE IF NOT EXISTS replies (
            id INTEGER PRIMARY KEY,
            confession_id INTEGER NOT NULL,
            text TEXT NOT NULL,
            name TEXT,
            author_fingerprint TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (confession_id) REFERENCES confessions(id)
        );",
    )?;
    Ok(conn)
}
