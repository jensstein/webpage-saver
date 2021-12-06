mod auth;
mod errors;

use std::path::Path;
use std::str::FromStr;

use clap::{App,Arg,ArgMatches};
use html5ever::tendril::TendrilSink;
use serde::Deserialize;
use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions,SqlitePool};
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

async fn write_to_db(conn: SqlitePool, url: &str, text: &str, html: &str, user_id: i64) ->
        Result<(), sqlx::Error> {
    let result = sqlx::query("INSERT INTO webpages(url, text, html, user_id) VALUES (?1, ?2, ?3, ?4)")
        .bind(url)
        .bind(text)
        .bind(html)
        .bind(user_id)
        .execute(&conn).await?;
    println!("ASD {} {:?}", url, result);
    Ok(())
}

async fn fetch_handler(db_pool: SqlitePool,
        http_client: reqwest::Client, body: FetchWebpage, user_id: i64) ->
        Result<impl warp::Reply, warp::Rejection> {

    match fetch_webpage(http_client, &body.url).await {
        Ok(html) => {
            let text = traverse_document(&html);
            match write_to_db(db_pool, &body.url, &text, &html, user_id).await {
                // Return-typen bestemmes af Responsens body, så hvis den er String det ene sted,
                // skal den også være String det andet. Og den er nødt til at være String i
                // Err-delen, fordi man ikke kan sende en reference til error ud af funktionen.
                Ok(_) => return Ok(Response::builder().status(StatusCode::OK).body("".to_string())),
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

async fn migrate_db(migrations_path: &str, conn: &SqlitePool) -> Result<(), sqlx::Error> {
    // The migration files must be called {VERSION}_{DESCRIPTION}.sql
    // https://docs.rs/sqlx-core/0.5.9/src/sqlx_core/migrate/source.rs.html#10-12
    let migrator = sqlx::migrate::Migrator::new(
        Path::new(migrations_path)).await?;
    migrator.run(conn).await.expect("Error running migrations");
    Ok(())
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

fn validate_int_arg(v: String) -> Result<(), String> {
    match v.parse::<u16>() {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("Error parsing {} as u16: {}", v, error.to_string()))
    }
}

fn setup_args() -> ArgMatches<'static> {
    App::new("article-server")
        .arg(Arg::with_name("port")
            .short("-p")
            .long("--port")
            .help("Port to start service on")
            .validator(validate_int_arg)
            .default_value("5000"))
        .arg(Arg::with_name("database-path")
            .long("--db-path")
            .help("Path to the database to store webpages in")
            .default_value("webpages.db"))
    .get_matches()
}

async fn create_sqlite_db(db_url: &str) -> Result<(), sqlx::Error> {
    SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .connect()
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = setup_args();
    let port = args.value_of("port").expect("Unable to get port argument")
        .parse::<u16>().expect("Unable to parse port argument");
    let db_url = args.value_of("database-path")
        .expect("Unable to get database-path argument");
    create_sqlite_db(db_url).await.expect("Error creating database file");
    let pool_ = SqlitePool::connect(db_url).await.expect("Unable to get database connection pool");
    migrate_db("db/migrations", &pool_).await.expect("Unable to migrate database");
    // This db pool is passed to the jwt authorization filter
    let auth_pool = pool_.clone();
    let pool = warp::any().map(move|| pool_.clone());
    let http_client = warp::any().map(move|| reqwest::Client::new());
    let api_routes = warp::post()
            .and(warp::path("fetch"))
            .and(pool.clone())
            .and(http_client)
            .and(warp::body::json())
            .and(auth::with_jwt_auth(auth_pool))
            .and_then(fetch_handler)
        .or(
            warp::get().and(warp::path("status")).map(|| "OK"))
        .or(warp::post()
            .and(warp::path("register"))
            .and(pool.clone())
            .and(warp::body::json())
            .and_then(auth::register_handler))
        .or(warp::post()
            .and(warp::path("login"))
            .and(pool.clone())
            .and(warp::body::json())
            .and_then(auth::login_handler))
        .or(warp::post()
            .and(warp::path("verify-jwt"))
            .and(pool.clone())
            .and(warp::body::json())
            .and_then(auth::verify_jwt_handler));
    let routes = warp::path("api").and(api_routes).recover(errors::handle_rejection);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

#[cfg(test)]
mod test_main;
