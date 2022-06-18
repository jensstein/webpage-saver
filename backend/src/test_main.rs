use super::*;

use test_utils::create_db;

use sqlx::Row;

#[derive(Debug)]
struct WebpagesRow {
    id: i64,
    url: String,
    title: String,
    text: String,
    html: String,
    image_url: Option<String>,
    user_id: i64,
    added: chrono::DateTime<chrono::Local>,
}

impl std::cmp::PartialEq for WebpagesRow {
    fn eq(&self, other: &Self) -> bool {
        (self.id == other.id) &&
        (self.url == other.url) && (self.text == other.text) &&
            (self.html == other.html) && (self.title == other.title) &&
            (self.image_url == other.image_url) && (self.user_id == self.user_id)
    }
}

#[test]
fn test_traverse_document() {
    let html = include_str!("test-data/file.html");
    let result = traverse_document(html);
    assert_eq!(result.title, "HUR".to_string());
    assert_eq!(result.contents, "<title > HUR </title> <h1 > overskrift </h1> <h2 > underoverskrift med <a href=link1> link </a> </h2> <img src=image.url> </img> <p > tekst 1 </p> <p > tekst med <a href=link2> link </a> og tekst </p> <p id=10> tekst med <span id=20> tekst inde i <span id=30> tekst </span> </span> </p>");
    assert_eq!(result.image_url, Some("image.url".to_string()));
}

#[tokio::test]
async fn test_database_migration() {
    let conn = create_db().await;
    migrate_db(&conn).await.expect("Unable to migrate database");
    sqlx::query("INSERT INTO users VALUES(1, 'user', '$argon2id$v=19$m=4096,t=3,p=1$ewSM8Hmctto5QHVv27S1cA$o6GeMd3PriFhi2CalkBmG1cV/AMi+ry0r/6fjmeSaFQ', '{user}')")
        .execute(&conn)
        .await.expect("Unable to insert into database");
    let before = chrono::Local::now();
    sqlx::query("INSERT INTO webpages(url, text, html, user_id, title, image_url) VALUES('url', 'text', 'html', 1, 'title', 'image.url')")
        .execute(&conn)
        .await.expect("Unable to insert into database");
    let after = chrono::Local::now();
    let rows = sqlx::query("SELECT * FROM webpages")
        .map(|row: sqlx::postgres::PgRow| {
            WebpagesRow {
                id: row.get(0),
                url: row.get(1),
                text: row.get(2),
                html: row.get(3),
                user_id: row.get(4),
                title: row.get(5),
                image_url: row.get(6),
                added: row.get(7),
            }
        })
        .fetch_all(&conn).await.expect("Error fetching rows from database");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], WebpagesRow{
        id: 1,
        url: "url".to_string(),
        text: "text".to_string(),
        html: "html".to_string(),
        user_id: 1,
        title: "title".to_string(),
        image_url: Some("image.url".to_string()),
        added: chrono::Local::now(),
    });
    assert_eq!(rows[0].added > before && rows[0].added < after, true);
}
