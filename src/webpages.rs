use crate::errors;

use serde::Deserialize;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use warp::http::StatusCode;

#[derive(Serialize,Debug)]
struct ShowWebpageResponse {
    title: String,
    image_url: Option<String>,
    content: String,
}

#[derive(Serialize,Debug)]
struct ListWebpagesResponse {
    webpage_infos: Vec<WebpageInfo>,
}

#[derive(Serialize,Debug)]
struct WebpageInfo {
    id: i64,
    title: String,
    image_url: Option<String>,
}

// The names of these enum variants should appear exactly as they are meant to be typed in as query
// parameters by the user.
#[allow(non_camel_case_types)]
#[derive(Deserialize,Debug)]
enum ShowMode {
    readable,
    original,
}

#[derive(Deserialize,Debug)]
pub struct ShowOptions {
    mode: Option<ShowMode>,
}

fn showmode_to_db_table(mode: &ShowMode) -> &str {
    match mode {
        ShowMode::readable => "text",
        ShowMode::original => "html",
    }
}

pub async fn show_stored_webpage_handler(webpage_id: i64, query_params: ShowOptions, db_pool: SqlitePool, user_id: i64) ->
        Result<impl warp::Reply, warp::Rejection> {
    let mode = &query_params.mode.unwrap_or(ShowMode::readable);
    let table = showmode_to_db_table(mode);
    let (response, status_code) = sqlx::query_as::<_, (String,Option<String>,String)>(&format!("SELECT title, image_url, {} FROM webpages WHERE rowid = $1 AND user_id = $2", table))
        .bind(webpage_id)
        .bind(user_id)
        .fetch_optional(&db_pool).await
        .map_or_else(|error| {
            eprintln!("Error when fetching webpage {}  for user {} from database {}",
                webpage_id, user_id, error);
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let json = warp::reply::json(&errors::ErrorResponse {
                message: "Unknown error".to_string(),
                status: status.to_string(),
            });
            (json, status)
        }, |optional_webpage| {
            match optional_webpage {
                Some((title, image_url, text)) => {
                    let json = warp::reply::json(&ShowWebpageResponse {
                        title,
                        image_url,
                        content: text
                    });
                    (json, StatusCode::OK)
                },
                None => {
                    let status = StatusCode::NOT_FOUND;
                    let json = warp::reply::json(&errors::ErrorResponse {
                        message: "Webpage not found".to_string(),
                        status: status.to_string()
                    });
                    (json, status)
                }
            }
        });
    Ok(warp::reply::with_status(response, status_code))
}

pub async fn get_stored_webpages_for_user(db_pool: SqlitePool, user_id: i64) ->
        Result<impl warp::Reply, warp::Rejection> {
    let (response, status_code) = sqlx::query_as::<_, (i64,String,Option<String>)>("SELECT rowid, title, image_url FROM webpages WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&db_pool).await
        .map_or_else(|error| {
            eprintln!("Error when fetching list of webpages for user {}: {}",
                user_id, error);
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let json = warp::reply::json(&errors::ErrorResponse {
                message: "Unknown error".to_string(),
                status: status.to_string(),
            });
            (json, status)
        }, |rows| {
            let webpage_infos = rows.iter().map(|(id, title, image_url)| {
                WebpageInfo {
                    id: id.to_owned(),
                    title: title.to_owned(),
                    image_url: image_url.to_owned()
                }
            }).collect::<Vec<WebpageInfo>>();
            let json = warp::reply::json(&ListWebpagesResponse {
                webpage_infos,
            });
            (json, StatusCode::OK)
        });
    Ok(warp::reply::with_status(response, status_code))
}
