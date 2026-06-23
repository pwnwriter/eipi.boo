use rusqlite::{Connection, Result as SqlResult, params};

use crate::model::confession::Confession;

pub fn get_all(conn: &Connection) -> Vec<Confession> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.text, c.x, c.y, c.votes,
                    (SELECT COUNT(*) FROM replies r WHERE r.confession_id = c.id),
                    c.created_at
             FROM confessions c ORDER BY c.id",
        )
        .unwrap();

    stmt.query_map([], |row| {
        Ok(Confession {
            id: row.get(0)?,
            text: row.get(1)?,
            x: row.get(2)?,
            y: row.get(3)?,
            votes: row.get(4)?,
            reply_count: row.get(5)?,
            created_at: row.get(6)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

pub fn insert(
    conn: &Connection,
    text: &str,
    x: i64,
    y: i64,
    fingerprint: &str,
) -> SqlResult<Confession> {
    conn.execute(
        "INSERT INTO confessions (text, x, y, author_fingerprint) VALUES (?1, ?2, ?3, ?4)",
        params![text, x, y, fingerprint],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Confession {
        id,
        text: text.to_string(),
        x,
        y,
        votes: 0,
        reply_count: 0,
        created_at: chrono::Utc::now()
            .naive_utc()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    })
}

pub fn posts_today(conn: &Connection, fingerprint: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM confessions
         WHERE author_fingerprint = ?1
         AND created_at > datetime('now', '-1 day')",
        params![fingerprint],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

pub fn stats(conn: &Connection) -> (i64, i64) {
    let confessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM confessions", [], |row| row.get(0))
        .unwrap_or(0);
    let humans: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT author_fingerprint) FROM confessions",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    (confessions, humans)
}
