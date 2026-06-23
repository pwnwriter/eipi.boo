use rusqlite::{Connection, Result as SqlResult, params};

use crate::model::reply::Reply;

pub fn get_replies(conn: &Connection, confession_id: i64) -> Vec<Reply> {
    let mut stmt = conn
        .prepare(
            "SELECT text, COALESCE(name, 'anon')
             FROM replies WHERE confession_id = ?1 ORDER BY id",
        )
        .unwrap();
    stmt.query_map(params![confession_id], |row| {
        Ok(Reply {
            text: row.get(0)?,
            name: row.get(1)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

pub fn insert_reply(
    conn: &Connection,
    confession_id: i64,
    text: &str,
    name: Option<&str>,
    fingerprint: &str,
) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO replies (confession_id, text, name, author_fingerprint) VALUES (?1, ?2, ?3, ?4)",
        params![confession_id, text, name, fingerprint],
    )?;
    Ok(())
}
