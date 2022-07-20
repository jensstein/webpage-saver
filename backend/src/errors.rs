use serde::Serialize;
use warp::http::StatusCode;

// Largely taken from https://github.com/zupzup/rust-jwt-example/blob/main/src/error.rs

#[derive(Debug)]
pub enum Error {
    MissingAuthorizationHeader,
    UnknownUser,
    UserMissingRole,
    OAuth2ProviderNotConfigured,
    OAuth2ProviderError,
}

impl warp::reject::Reject for Error {}

#[derive(Serialize,Debug)]
pub struct ErrorResponse {
    pub message: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ParseDocumentError<'a> {
    message: &'a str,
}

impl std::fmt::Display for ParseDocumentError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseDocumentError<'_> {}
impl<'a> ParseDocumentError<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {message}
    }
}

pub async fn handle_rejection(rejection: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(error) = rejection.find::<Error>() {
        let (message, status_code) = match error {
            Error::MissingAuthorizationHeader => ("Missing or malformed authorization header", StatusCode::UNAUTHORIZED),
            Error::UnknownUser => ("Unknown user", StatusCode::UNAUTHORIZED),
            Error::UserMissingRole => ("User missing role", StatusCode::UNAUTHORIZED),
            Error::OAuth2ProviderNotConfigured => ("OAuth2 not allowed", StatusCode::UNAUTHORIZED),
            Error::OAuth2ProviderError => ("OAuth2 not allowed", StatusCode::UNAUTHORIZED),
        };
        let json = warp::reply::json(&ErrorResponse {
            message: message.to_string(),
            status: status_code.to_string(),
        });
        return Ok(warp::reply::with_status(json, status_code));
    }
    Err(rejection)
}
