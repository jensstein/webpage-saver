use super::*;

use sqlx::sqlite::SqlitePool;
use sqlx::Row;

#[derive(Debug)]
struct WebpagesRow {
    url: String,
    text: String,
    html: String,
    user_id: i64
}

impl std::cmp::PartialEq for WebpagesRow {
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
    sqlx::query("INSERT INTO users VALUES(1, 'user', '$argon2id$v=19$m=4096,t=3,p=1$ewSM8Hmctto5QHVv27S1cA$o6GeMd3PriFhi2CalkBmG1cV/AMi+ry0r/6fjmeSaFQ')")
        .execute(&conn)
        .await.expect("Unable to insert into database");
    sqlx::query("INSERT INTO webpages VALUES('url', 'text', 'html', 1)")
        .execute(&conn)
        .await.expect("Unable to insert into database");
    let rows = sqlx::query("SELECT * FROM webpages")
        .map(|row: sqlx::sqlite::SqliteRow| {
            WebpagesRow {
                url: row.get(0),
                text: row.get(1),
                html: row.get(2),
                user_id: row.get(3)
            }
        })
        .fetch_all(&conn).await.expect("Error fetching rows from database");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], WebpagesRow{url: "url".to_string(),
        text: "text".to_string(), html: "html".to_string(), user_id: 1});
}
