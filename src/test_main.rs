use super::*;

use rusqlite::Connection;

#[derive(Debug)]
struct DBRow {
    url: String,
    text: String,
    html: String,
}

impl std::cmp::PartialEq for DBRow {
    fn eq(&self, other: &Self) -> bool {
        (self.url == other.url) && (self.text == other.text) && (self.html == other.html)
    }
}

#[test]
fn test_traverse_document() {
    let html = include_str!("test-data/file.html");
    let result = traverse_document(html);
    assert_eq!(result, "<title > HUR </title> <h1 > overskrift </h1> <h2 > underoverskrift med <a href=link1> link </a> </h2> <p > tekst 1 </p> <p > tekst med <a href=link2> link </a> og tekst </p> <p id=10> tekst med <span id=20> tekst inde i <span id=30> tekst </span> </span> </p>");
}

#[test]
fn test_database_migration() {
    let mut conn = Connection::open_in_memory()
        .expect("Unable to open database in memory");
    migrate_db(&mut conn).expect("Unable to migrate database");
    conn.execute("INSERT INTO webpages VALUES('url', 'text', 'html')", [])
        .expect("Unable to insert into database");
    let mut sql = conn.prepare("SELECT * FROM webpages")
        .expect("Unable to prepare sql statement");
    let rows = sql.query_map([], |row| Ok(DBRow {
        url: row.get(0).expect("Unable to get column 0 of row"),
        text: row.get(1).expect("Unable to get column 1 of row"),
        html: row.get(2).expect("Unable to get column 2 of row")
    })).expect("Unable to get rows from database")
        .map(|row| row.expect("Unable to fetch all columns of row"))
        .collect::<Vec<DBRow>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], DBRow{url: "url".to_string(),
        text: "text".to_string(), html: "html".to_string()});
}
