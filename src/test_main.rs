use super::*;

use sqlx::sqlite::SqlitePool;
use sqlx::Row;

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

#[tokio::test]
async fn test_database_migration() {
    let conn = SqlitePool::connect("sqlite::memory:").await
        .expect("Unable to open database in memory");
    migrate_db("db/migrations", &conn).await.expect("Unable to migrate database");
    sqlx::query("INSERT INTO webpages VALUES('url', 'text', 'html')")
        .execute(&conn)
        .await.expect("Unable to insert into database");
    let rows = sqlx::query("SELECT * FROM webpages")
        .map(|row: sqlx::sqlite::SqliteRow| {
            DBRow {
                url: row.get(0),
                text: row.get(1),
                html: row.get(2)
            }
        })
        .fetch_all(&conn).await.expect("Error fetching rows from database");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], DBRow{url: "url".to_string(),
        text: "text".to_string(), html: "html".to_string()});
}
