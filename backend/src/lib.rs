pub mod auth;
mod errors;
pub mod webpages;

use std::net::SocketAddr;
use std::sync::Arc;

use clap::{App,Arg,ArgMatches};
use html5ever::tendril::TendrilSink;
use kuchiki::iter::NodeIterator;
use serde::{Deserialize,Serialize};
use sqlx::PgPool;
use warp::Filter;
use warp::http::{StatusCode,Response};

// Using the migrate! macro embeds the migrations into the binary file
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("db/migrations");

// https://blog.joco.dev/posts/warp_auth_server_tutorial/

#[derive(Deserialize,Debug)]
struct FetchWebpage {
    url: String,
    html: Option<String>,
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

async fn fetch_webpage(http_client: &reqwest::Client, url: &str) -> Result<String, FetchError> {
    match http_client.get(url).send().await {
        Ok(response) => {
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

async fn write_to_db(conn: &PgPool, url: &str, webpage: &Webpage,  user_id: i64) ->
        Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    Ok(sqlx::query("INSERT INTO webpages(url, title, text, html, image_url, user_id) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(url)
        .bind(&webpage.title)
        .bind(&webpage.contents)
        .bind(&webpage.original_html)
        .bind(&webpage.image_url)
        .bind(user_id)
        .execute(conn).await?)
}

async fn fetch_handler(db_pool: Arc<PgPool>,
        http_client: reqwest::Client, body: FetchWebpage, user_id: i64) ->
        Result<impl warp::Reply, warp::Rejection> {

    let html = match body.html {
        Some(html) => html,
        None => match fetch_webpage(&http_client, &body.url).await {
            Ok(html) => html,
            Err(error) => {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(error.to_string())
                );
            }
        }
    };
    match traverse_document(&html) {
        Ok(webpage) => {
            match write_to_db(&db_pool, &body.url, &webpage, user_id).await {
                // Return-typen bestemmes af Responsens body, så hvis den er String det ene sted,
                // skal den også være String det andet. Og den er nødt til at være String i
                // Err-delen, fordi man ikke kan sende en reference til error ud af funktionen.
                Ok(_) => return Ok(Response::builder().status(StatusCode::CREATED).body("".to_string())),
                Err(error) => {
                    log::error!("Error getting database connection: {}", error.to_string());
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("".to_string())
                    );
                }
            }
        },
        Err(error) => {
            log::error!("Error parsing document for url {}: {}", &body.url, error);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()))
        }
    }
}

#[derive(Serialize,Debug)]
struct UserInfo {
    username: String,
}

async fn userinfo_handler(db_pool: Arc<PgPool>, user_id: i64) ->
        Result<impl warp::Reply, warp::Rejection> {
    sqlx::query_as::<_, (String,)>("select username from users where id = $1")
            .bind(user_id)
            .fetch_optional(&*db_pool).await
            .map_or_else(|error| {
                log::error!("Error when fetching user from database: {}", error);
                return Err(warp::reject::custom(errors::Error::UnknownUser))
            }, |optional_user| {
                optional_user.ok_or_else(|| warp::reject::custom(errors::Error::UnknownUser))
            })
            .and_then(|(username,)| {
                let json = warp::reply::json(&UserInfo {
                    username,
                });
                Ok(warp::reply::with_status(json, StatusCode::OK))
            })
}

pub async fn migrate_db(conn: &PgPool) -> Result<(), sqlx::Error> {
    Ok(MIGRATOR.run(conn).await?)
}

#[derive(Debug)]
struct Webpage {
    title: String,
    contents: String,
    image_url: Option<String>,
    original_html: String,
}

fn traverse_document(html: &str) -> Result<Webpage, errors::ParseDocumentError> {
    let document = kuchiki::parse_html().one(html);
    // https://stackoverflow.com/a/66277475
    document.inclusive_descendants()
        .filter(|node| node.as_element().map_or(false, |e| {
            matches!(e.name.local.as_ref(), "script" | "style" | "noscript")
        }))
        .collect::<Vec<_>>()
        .iter()
        .for_each(|node| node.detach());
    let title = document.select_first("title").map_or_else(|_| {
            let mut i = 0;
            let mut string_builder = Vec::new();
            for child in document.inclusive_descendants().text_nodes() {
                string_builder.push(child.borrow().to_string());
                if i >= 5 {
                    break;
                }
                i += 1;
            }
            string_builder.join(" ").trim().to_string()
        }, |node| node.text_contents().trim().to_string());
    let image_url = document.select_first("img").map_or(None, |node| {
        match node.as_node().data() {
            kuchiki::NodeData::Element(element_data) => {
                for (key, value) in &element_data.attributes.borrow().map {
                    if key.local.to_string() == "src" {
                        return Some(value.value.to_string())
                    }
                }
                None
            },
            _ => None
        }
    });
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
    let contents = string_builder.join(" ");
    if !contents.is_empty() {
        Ok(Webpage {
            title,
            contents,
            image_url,
            original_html: html.to_string(),
        })
    } else {
        Err(errors::ParseDocumentError::new("Unable to extract any text from document"))
    }
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

fn validate_host_arg(v: String) -> Result<(), String> {
    match format!("{}:0", v).parse::<SocketAddr>() {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("Error parsing {} as a hostname: {}", v, error))
    }
}

pub fn setup_args() -> ArgMatches<'static> {
    App::new("article-server")
        .arg(Arg::with_name("host")
            .long("--host")
            .help("Host address to start service on")
            .validator(validate_host_arg)
            .default_value("0.0.0.0"))
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

#[derive(Clone)]
pub struct ServerArgs {
    pub pool: PgPool,
    pub addr: SocketAddr,
}

pub fn start_server(args: ServerArgs) -> impl std::future::Future<Output = ()> + 'static {
    // Wrap the pool object in a reference counter to avoid cloning the actual object every time it
    // is passed to a handler. The warp filters cannot take &-references but an Arc can be used to
    // avoid cloning the actual object. This way only the Arc is cloned on every call to .clone().
    // All the different Arcs will still point to the same allocation on the heap.
    // https://rust-unofficial.github.io/patterns/anti_patterns/borrow_clone.html
    // https://doc.rust-lang.org/std/sync/struct.Arc.html
    // https://doc.rust-lang.org/rust-by-example/std/arc.html
    // https://stackoverflow.com/a/71191133
    // https://github.com/seanmonstar/warp/blob/master/examples/todos.rs
    let pool = Arc::new(args.pool);
    // This db pool is passed to the jwt authorization filter. It needs to be a regular object and
    // not a warp filter so the pool object passed to the handlers cannot be used here.
    let auth_pool = pool.clone();
    let pool = warp::any().map(move|| pool.clone());
    let http_client = warp::any().map(move|| reqwest::Client::new());
    let api_routes = warp::post()
            .and(warp::path("fetch"))
            .and(pool.clone())
            .and(http_client)
            .and(warp::body::json())
            .and(auth::with_jwt_auth(auth_pool.clone(), vec![auth::Role::User, auth::Role::Admin]))
            .and_then(fetch_handler)
        .or(
            warp::get().and(warp::path("status")).map(|| "OK"))
        .or(warp::get()
            .and(warp::path("webpage"))
            .and(warp::path::param())
            .and(warp::query::<webpages::ShowOptions>())
            .and(pool.clone())
            .and(auth::with_jwt_auth(auth_pool.clone(), vec![auth::Role::User, auth::Role::Admin]))
            .and_then(webpages::show_stored_webpage_handler))
        .or(warp::delete()
            .and(warp::path("webpage"))
            .and(warp::path::param())
            .and(pool.clone())
            .and(auth::with_jwt_auth(auth_pool.clone(), vec![auth::Role::User, auth::Role::Admin]))
            .and_then(webpages::delete_stored_webpage_handler))
        .or(warp::get()
            .and(warp::path("list-stored-webpages"))
            .and(pool.clone())
            .and(auth::with_jwt_auth(auth_pool.clone(), vec![auth::Role::User, auth::Role::Admin]))
            .and_then(webpages::get_stored_webpages_for_user))
        .or(warp::post()
            .and(warp::path("register"))
            .and(pool.clone())
            .and(warp::body::json())
            .and(auth::with_jwt_auth(auth_pool.clone(), vec![auth::Role::Admin]))
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
            .and_then(auth::verify_jwt_handler))
        .or(warp::get()
            .and(warp::path("extend-jwt"))
            .and(pool.clone())
            .and(auth::with_jwt_auth(auth_pool.clone(), vec![auth::Role::User, auth::Role::Admin]))
            .and_then(auth::extend_jwt_handler))
        .or(warp::post()
            .and(warp::path("associate-app-to-user"))
            .and(pool.clone())
            .and(auth::with_jwt_auth(auth_pool.clone(), vec![auth::Role::User, auth::Role::Admin]))
            .and(warp::body::json())
            .and_then(auth::associate_app_to_user_handler))
        .or(warp::get()
            .and(warp::path("userinfo"))
            .and(pool.clone())
            .and(auth::with_jwt_auth(auth_pool, vec![auth::Role::User, auth::Role::Admin]))
            .and_then(userinfo_handler));
    let routes = warp::path("api").and(api_routes.with(
        warp::log("article-saver"))).recover(errors::handle_rejection);
    warp::serve(routes).bind(args.addr)
}

#[cfg(test)]
mod test_main;
