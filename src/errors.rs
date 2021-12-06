use serde::Serialize;
use warp::http::StatusCode;

// Largely taken from https://github.com/zupzup/rust-jwt-example/blob/main/src/error.rs

#[derive(Debug)]
pub enum Error {
    MissingAuthorizationHeader,
    UnknownUser,
}

impl warp::reject::Reject for Error {}

#[derive(Serialize,Debug)]
pub struct ErrorResponse {
    pub message: String,
    pub status: String,
}

pub async fn handle_rejection(rejection: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(error) = rejection.find::<Error>() {
        let (message, status_code) = match error {
            Error::MissingAuthorizationHeader => ("Missing or malformed authorization header", StatusCode::UNAUTHORIZED),
            Error::UnknownUser => ("Unknown user", StatusCode::UNAUTHORIZED),
        };
        let json = warp::reply::json(&ErrorResponse {
            message: message.to_string(),
            status: status_code.to_string(),
        });
        return Ok(warp::reply::with_status(json, status_code));
    }
    Err(rejection)
}
