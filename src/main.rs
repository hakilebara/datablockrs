use rusqlite::{params, Connection};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let conn = Connection::open_in_memory()?;
    let mut stmt =
        conn.prepare("CREATE TABLE blocks (id TEXT PRIMARY KEY, title TEXT NOT NULL)")?;
    stmt.execute([])?;

    let uuid = Uuid::new_v4();
    stmt = conn.prepare("INSERT INTO blocks (id, title) VALUES (?1, ?2)")?;
    stmt.execute(params![uuid.to_string(), "hello"])?;

    let title: String = conn.query_one(
        "SELECT id, title FROM blocks WHERE title == 'hello'",
        [],
        |row| row.get(1),
    )?;
    println!("{}", title);
    Ok(())
}
