use html5ever::tendril::TendrilSink;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;
use warp::Filter;
use warp::http::{StatusCode,Response};

// https://blog.joco.dev/posts/warp_auth_server_tutorial/

#[derive(Deserialize,Debug)]
struct FetchWebpage {
    url: String,
}

#[derive(Debug, Clone)]
struct FetchError {
    message: String,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FetchError {}

impl From<reqwest::Error> for FetchError {
    fn from(error: reqwest::Error) -> Self {
        FetchError {message: error.to_string()}
    }
}

async fn fetch_webpage(http_client: reqwest::Client, url: &str) -> Result<String, FetchError> {
    match http_client.get(url).send().await {
        Ok(response) => {
            println!("Result {:?}", response);
            if response.status().is_success() {
                return Ok(response.text().await?);
            } else {
                return Err(FetchError {
                    message: format!("Unable to fetch {}. Got status {}: {}",
                        url, response.status(), response.text().await?)
                });
            }
        },
        Err(error) => {
            return Err(FetchError{message: format!("Unable to fetch {}. Error: {}", url, error.to_string())});
        }
    }
}

fn write_to_db(conn: r2d2::PooledConnection<SqliteConnectionManager>, url: &str, text: &str, html: &str) {
    let result = conn.execute("INSERT INTO webpages(url, text, html) VALUES (?1, ?2, ?3)", rusqlite::params![url, text, html]);
    println!("ASD {} {:?}", url, result);
}

async fn fetch_handler(db_pool: r2d2::Pool<SqliteConnectionManager>,
        http_client: reqwest::Client, body: FetchWebpage) ->
        Result<impl warp::Reply, warp::Rejection> {
    match fetch_webpage(http_client, &body.url).await {
        Ok(html) => {
            match db_pool.get() {
                Ok(conn) => {
                    let text = traverse_document(&html);
                    write_to_db(conn, &body.url, &text, &html);
                    // Return-typen bestemmes af Responsens body, så hvis den er String det ene sted,
                    // skal den også være String det andet. Og den er nødt til at være String i
                    // Err-delen, fordi man ikke kan sende en reference til error ud af funktionen.
                    return Ok(Response::builder().status(StatusCode::OK).body("".to_string()));
                },
                Err(error) => {
                    eprintln!("Error getting database connection: {}", error.to_string());
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("".to_string())
                    );
                }
            }
        },
        Err(error) => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(error.to_string())
            );
        }
    }
}

fn migrate_db(db_url: &str) -> Result<(), rusqlite_migration::Error> {
    let mut conn = rusqlite::Connection::open(db_url)?;
    let migrations = rusqlite_migration::Migrations::new(vec![
        rusqlite_migration::M::up("
            CREATE TABLE webpages(url TEXT NOT NULL, text TEXT NOT NULL, html TEXT NOT NULL);
            CREATE INDEX webpage_url_idx ON webpages(url);
        ")
    ]);
    migrations.to_latest(&mut conn)
}

fn traverse_document(html: &str) -> String {
    let document = kuchiki::parse_html().one(html);
    // https://stackoverflow.com/a/66277475
    document.inclusive_descendants()
        .filter(|node| node.as_element().map_or(false, |e| {
            matches!(e.name.local.as_ref(), "script" | "style" | "noscript")
        }))
        .collect::<Vec<_>>()
        .iter()
        .for_each(|node| node.detach());
    let tags_to_ignore = ["html", "head", "meta", "link", "style", "body",
        "main", "article", "div", "script", "nav", "ul", "footer", "svg",
        "path", "figure", "picture", "iframe"];
    let mut string_builder = Vec::new();
    for node_edge in document.traverse_inclusive() {
        match node_edge {
            kuchiki::iter::NodeEdge::Start(node) => {
                match node.data() {
                    kuchiki::NodeData::Element(element_data) => {
                        let tag = &element_data.name.local.to_string();
                        if tags_to_ignore.iter().all(|item| item != tag) {
                            let attributes = attributes_to_string(
                                &element_data.attributes.borrow());
                            string_builder.push(format!("<{} {}>", tag,
                                attributes));
                        }
                    },
                    kuchiki::NodeData::Text(value) => {
                        let text = value.borrow().trim().to_string();
                        if text != "" {
                            string_builder.push(text.to_string());
                        }
                    },
                    _ => {}
                }
            },
            kuchiki::iter::NodeEdge::End(node) => {
                match node.data() {
                    kuchiki::NodeData::Element(element_data) => {
                        let tag = &element_data.name.local.to_string();
                        if tags_to_ignore.iter().all(|item| item != tag) {
                            string_builder.push(format!("</{}>", tag));
                        }
                    },
                    _ => {}
                }
            },
        }
    }
    string_builder.join(" ")
}

fn attributes_to_string(attributes: &kuchiki::Attributes) -> String {
    let mut string_builder = Vec::with_capacity(attributes.map.len());
    for (key, value) in &attributes.map {
        string_builder.push(format!("{}={}", key.local, value.value));
    }
    string_builder.join(" ")
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let db_url = "file.db";
    let manager = SqliteConnectionManager::file(db_url);
    migrate_db(db_url).expect("Unable to migrate database schema");
    let pool_ = r2d2::Pool::new(manager).expect("Unable to get database connection pool");
    let pool = warp::any().map(move|| pool_.clone());
    let http_client = warp::any().map(move|| reqwest::Client::new());
    let routes = warp::post()
            .and(warp::path("fetch"))
            .and(pool)
            .and(http_client)
            .and(warp::body::json())
            .and_then(fetch_handler)
        .or(
            warp::get().and(warp::path("status")).map(|| "OK")
    );
    warp::serve(routes).run(([127, 0, 0, 1], 6000)).await;
}

#[cfg(test)]
mod test_main;
