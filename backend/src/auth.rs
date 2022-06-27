use crate::errors;

use argon2::password_hash::{PasswordHash,SaltString};
use argon2::{Argon2,PasswordHasher,PasswordVerifier};
use rand::Rng;
use sqlx::PgPool;
use serde::Deserialize;
use serde::Serialize;
use warp::Filter;
use warp::filters::header::headers_cloned;
use warp::http::{header::HeaderMap,header::HeaderValue,StatusCode,Response};

// https://www.lpalmieri.com/posts/password-authentication-in-rust/

// JWT authorization: https://blog.logrocket.com/jwt-authentication-in-rust/

#[derive(Debug,Clone)]
pub struct AuthError {
    message: String
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug,Clone)]
pub struct AuthConfigError {
    message: String
}

impl std::fmt::Display for AuthConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl From<jsonwebtoken::errors::Error> for AuthConfigError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AuthConfigError {message: error.to_string()}
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(error: argon2::password_hash::Error) -> Self {
        AuthError {message: error.to_string()}
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AuthError {message: error.to_string()}
    }
}

#[derive(Deserialize,Debug)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(sqlx::Type, Debug, Eq, PartialEq, Clone)]
#[sqlx(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}

// Enable sqlx to convert an array of the custom user_role type to a Vec<Role>
impl sqlx::postgres::PgHasArrayType for Role {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        // Why does it need an underscore in front of the name when that's not part of the type
        // definition in postgres?
        sqlx::postgres::PgTypeInfo::with_name("_user_role")
    }
}

#[derive(Deserialize,Debug)]
pub struct JWTRequest {
    pub username: String,
    pub jwt: String,
}

#[derive(Serialize,Debug)]
struct JWTResponse {
    jwt: String
}

fn validate_password_chars(password: &str) -> bool {
    password.len() >= 8
}

fn validate_username_chars(username: &str) -> bool {
    lazy_static::lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new("[\\s;]").expect("Compiling the regex failed");
    }
    !RE.is_match(username)
}

// https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-user-defined-characters
pub fn generate_random_string() -> Vec<u8> {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
        abcdefghijklmnopqrstuvwxyz\
        0123456789)(*&^%$#@!~";
    let mut rng = rand::thread_rng();
    (0..50).map(|_| {
        let idx = rng.gen_range(0..charset.len());
        charset[idx]
    })
    .collect()
}

async fn store_new_user_with_jwt_secret(db_pool: PgPool, username: &str, hashed_password: &str)
        -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let generated_jwt_secret = generate_random_string();
    Ok(sqlx::query("
WITH new_user AS (
    INSERT INTO users(username, password_hash, roles) VALUES($1, $2, '{user}') RETURNING id
    )
INSERT INTO jwt_secrets(secret, user_id) (SELECT $3, id FROM new_user)
")
        .bind(username)
        .bind(hashed_password)
        .bind(generated_jwt_secret)
        .execute(&db_pool).await?)
}

pub async fn register_handler(db_pool: PgPool, body: User, _admin_user_id: i64) ->
        Result<impl warp::Reply, warp::Rejection> {
    if !validate_password_chars(&body.password) {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Password must have at least 8 characters".to_string())
        )
    }
    if !validate_username_chars(&body.username) {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Username cannot contain [ ;]".to_string())
        )
    }
    let hashed_password = match hash_password(&body.password) {
        Ok(password) => password,
        Err(error) => {
            log::error!("Error hashing password for user {}: {}", &body.username, error.to_string());
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid password".to_string())
            )
        }
    };
    match store_new_user_with_jwt_secret(db_pool, &body.username, &hashed_password).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body("".to_string())),
        Err(error) => {
            log::error!("Error when storing user {}: {}", &body.username, error);
            return Ok(Response::builder()
                .status(StatusCode::CONFLICT)
                .body("".to_string())
            );
        }
    }
}

fn verify_user_with_password(provided_pass: &str, optional_hashed_pass: &Option<(i64, String, String)>) -> Result<Option<(i64, String)>, AuthError> {
    let fallback_password = "$argon2id$v=19$m=4096,t=3,p=1$ewSM8Hmctto5QHVv27S1cA$o6GeMd3PriFhi2CalkBmG1cV/AMi+ry0r/6fjmeSaFQ";
    match optional_hashed_pass {
        Some((user_id, password_hash, jwt_secret)) => {
            if verify_password(provided_pass, &password_hash)? {
                Ok(Some((user_id.to_owned(), jwt_secret.to_owned())))
            } else {
                Ok(None)
            }
        },
        None => {
            // This part is here to make sure that you can't analyse the timing of the response and
            // gather information about the database that way. It isn't a concrete risk right now
            // because you could get the same information by just trying to register an account,
            // but it doesn't hurt to have it.
            verify_password("", fallback_password)?;
            Ok(None)
        }
    }
}

pub async fn verify_password_from_database(db_pool: &PgPool, username: &str, password: &str)
        -> Result<Option<(i64, String)>, AuthError> {
    sqlx::query_as::<_, (i64, String, String)>("SELECT users.id, users.password_hash, jwt_secrets.secret FROM users JOIN jwt_secrets ON users.id = jwt_secrets.user_id WHERE username = $1")
            .bind(username)
            .fetch_optional(db_pool).await
            .map_or_else(|error| {
                Err(AuthError {message: format!("Error getting password from database for user {}: {}", username, error)})
            }, |optional_hashed_pass| {
                if let Some(user_id_and_secret) = verify_user_with_password(password, &optional_hashed_pass)? {
                    Ok(Some(user_id_and_secret))
                } else {
                    log::error!("Unable to fetch user info for user {}", username);
                    Ok(None)
                }
            })
}

pub async fn login_handler(db_pool: PgPool, body: User) ->
        Result<impl warp::Reply, warp::Rejection> {
    let (response, status_code) = verify_password_from_database(&db_pool, &body.username, &body.password)
        .await
        .map_or_else(|error| {
            log::error!("Error when verifying password for user {}: {}", &body.username, error);
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let json = warp::reply::json(&errors::ErrorResponse {
                message: "Unknown error".to_string(),
                status: status.to_string(),
            });
            (json, status)
            }, |verified_user_id| {
                    match verified_user_id {
                        Some((_, secret)) => {
                            match create_jwt(&body.username, secret.as_ref(), 60 * 60 * 24) {
                                Ok(jwt) => {
                                    let json = warp::reply::json(&JWTResponse {
                                        jwt
                                    });
                                    (json, StatusCode::OK)
                                },
                                Err(error) => {
                                    log::error!("Error when creating jwt for user {}: {}", &body.username, error);
                                    let status = StatusCode::INTERNAL_SERVER_ERROR;
                                    let json = warp::reply::json(&errors::ErrorResponse {
                                        message: "Unknown error".to_string(),
                                        status: status.to_string(),
                                    });
                                    (json, status)
                                }
                            }
                        },
                        None => {
                            let status = StatusCode::UNAUTHORIZED;
                            let json = warp::reply::json(&errors::ErrorResponse {
                                message: "Password doesn't match".to_string(),
                                status: status.to_string(),
                            });
                            (json, status)
                        }
                    }
            });
    Ok(warp::reply::with_status(response, status_code))
}

fn decode_jwt(jwt: &str, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Claims>(&jwt,
        &jsonwebtoken::DecodingKey::from_secret(secret),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512))
    .map(|decoded_jwt| decoded_jwt.claims.sub)
}

pub fn get_sub_from_jwt_insecure(jwt: &str) -> Option<String> {
    match jsonwebtoken::dangerous_insecure_decode_with_validation::<Claims>(
            &jwt, &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512)) {
        Ok(token) => Some(token.claims.sub),
        Err(error) => {
            log::error!("Error decoding jwt {}: {}", jwt, error);
            None
        }
    }
}

async fn verify_jwt(db_pool: &PgPool, username: &str, jwt: &str) ->
        Result<String, AuthError> {
    sqlx::query_as::<_, (String,)>("SELECT jwt_secrets.secret FROM jwt_secrets JOIN users ON jwt_secrets.user_id = users.id WHERE users.username = $1")
            .bind(username)
            .fetch_optional(db_pool).await
            .map_or_else(|error| {
                Err(AuthError {message: format!("Error fetching jwt secret for user {}: {}", username, error)})
        }, |optional_secret: Option<(String,)>| {
            match optional_secret {
                Some(secret) => Ok(decode_jwt(jwt, secret.0.as_bytes())?),
                None => Err(AuthError {message: format!("No jwt secret found for user {}", username)})
            }
        })
}

pub async fn verify_jwt_handler(db_pool: PgPool, jwt_body: JWTRequest)
        -> Result<impl warp::Reply, warp::Rejection> {
    match verify_jwt(&db_pool, &jwt_body.username, &jwt_body.jwt).await {
        Ok(_) => Ok(Response::builder()
                .status(StatusCode::OK)
                .body("".to_string())),
        Err(error) => {
            log::error!("Error when verifying jwt: {}", error);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("".to_string()))
        }
    }
}

// The naming of these fields is significant: https://github.com/Keats/jsonwebtoken#validation
#[derive(Debug,Deserialize,Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_jwt(username: &str, secret: &[u8], expiration_in_seconds: i64) -> Result<String, AuthConfigError> {
    let experiation = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expiration_in_seconds))
            .ok_or_else(|| AuthConfigError {
                message: format!("Error adding {} seconds to the current time", expiration_in_seconds)
            })?
        .timestamp() as usize;
    let claims = Claims {sub: username.to_string(), exp: experiation};
    Ok(jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512),
        &claims, &jsonwebtoken::EncodingKey::from_secret(secret)
    )?)
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    Ok(Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string())
}

fn verify_password(password: &str, hashed_password: &str) -> Result<bool, argon2::password_hash::Error> {
    let hash = PasswordHash::new(&hashed_password)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &hash).is_ok())
}

pub fn with_jwt_auth(db_pool: PgPool, roles: Vec<Role>) -> impl warp::Filter<Extract = (i64,), Error = warp::Rejection> + Clone {
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| (db_pool.clone(), roles.clone(), headers))
        .and_then(authorize_from_jwt)
}

async fn authorize_from_jwt(arg_tuple: (PgPool, Vec<Role>, HeaderMap)) -> Result<i64, warp::Rejection> {
    let (db_pool, requested_roles, headers) = arg_tuple;
    let header = match headers.get("Authorization") {
        Some(header_value) => header_value,
        None => return Err(warp::reject::custom(errors::Error::MissingAuthorizationHeader))
    };
    // The to_str method of HeaderValue can only parse as visible ascii characters so as_bytes is
    // used to get a larger range of possible characters.
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(auth_header) => auth_header,
        Err(error) => {
            log::error!("Error parsing auth header: {}", error);
            return Err(warp::reject::custom(errors::Error::MissingAuthorizationHeader));
        }
    };
    if !auth_header.to_lowercase().starts_with("bearer ") {
        return Err(warp::reject::custom(errors::Error::MissingAuthorizationHeader));
    }
    let jwt = auth_header[7..].to_string();
    let sub = get_sub_from_jwt_insecure(&jwt);
    sqlx::query_as::<_, (i64, Vec<Role>, String)>("SELECT users.id, users.roles, secret FROM jwt_secrets JOIN users ON users.id = jwt_secrets.user_id WHERE users.username = $1")
            .bind(sub)
            .fetch_optional(&db_pool).await
            .map_or_else(|error| {
                log::error!("Error when fetching jwt secret from database: {}", error);
                return Err(warp::reject::custom(errors::Error::UnknownUser))
            }, |optional_secret| {
                optional_secret.ok_or_else(|| warp::reject::custom(errors::Error::UnknownUser))
            })
            .and_then(|(user_id, roles, secret)| {
                decode_jwt(&jwt, secret.as_bytes())
                    .map_or_else(
                        |error| {
                            log::error!("Error when decoding jwt: {}", error);
                            Err(warp::reject::custom(errors::Error::MissingAuthorizationHeader))
                        },
                        |_| {
                            if !roles.iter().any(|item| requested_roles.contains(item)) {
                                return Err(warp::reject::custom(errors::Error::UserMissingRole));
                            }
                            Ok(user_id)
                        })
            })
}

pub async fn extend_jwt_handler(db_pool: PgPool, user_id: i64) ->
        Result<impl warp::Reply, warp::Rejection> {
    let (response, status_code) = sqlx::query_as::<_, (String, Vec<u8>)>("SELECT users.username, jwt_secrets.secret FROM users JOIN jwt_secrets ON users.id = jwt_secrets.user_id WHERE users.id = $1")
        .bind(user_id)
        .fetch_optional(&db_pool).await
        .map_or_else(|error| {
            log::error!("Error when creating jwt for user {}: {}", user_id, error);
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let json = warp::reply::json(&errors::ErrorResponse {
                message: format!("Error creating jwt"),
                status: status.to_string(),
            });
            (json, status)
        }, |opt_row| {
            match opt_row {
                Some((username, secret)) => {
                    match create_jwt(&username, secret.as_ref(), 60 * 60 * 24) {
                        Ok(jwt) => {
                            let json = warp::reply::json(&JWTResponse {
                                jwt
                            });
                            (json, StatusCode::OK)
                        },
                        Err(error) => {
                            log::error!("Error when creating jwt for user {}: {}", username, error);
                            let status = StatusCode::INTERNAL_SERVER_ERROR;
                            let json = warp::reply::json(&errors::ErrorResponse {
                                message: format!("Error creating jwt: {}", error),
                                status: status.to_string(),
                            });
                            (json, status)
                        }
                    }
                },
                None => {
                    let status = StatusCode::UNAUTHORIZED;
                    let json = warp::reply::json(&errors::ErrorResponse {
                        message: "Unknown user".to_string(),
                        status: status.to_string(),
                    });
                    (json, status)
                }
            }
        });
    Ok(warp::reply::with_status(response, status_code))
}

#[derive(Deserialize,Debug)]
pub struct AppInfo {
    pub sub: String,
    pub client_id: String,
    pub app_host: String
}

pub async fn associate_app_to_user_handler(db_pool: PgPool, user_id: i64, appinfo: AppInfo) ->
        Result<impl warp::Reply, warp::Rejection> {
    let mut app_host = appinfo.app_host;
    // Before trying to store the values we'll have to check if a user_id->app_host binding already
    // exist. If it does then the app_host value will have the current time appended to ensure
    // uniqueness.
    match sqlx::query_as::<_, (String,)>("select app_host from connected_apps where user_id = $1")
            .bind(user_id)
            .fetch_all(&db_pool).await {
        Ok(app_hosts) => {
            for s in &app_hosts {
                if s.0 == app_host {
                    let now = chrono::offset::Utc::now();
                    app_host = format!("{}-{}", app_host, now.format("%Y-%m-%dT%H:%M:%S"));
                    break;
                }
            }
        },
        Err(error) => {
            log::error!("Error getting app hosts for user {}: {}",
                user_id, error);
            return Ok(warp::reply::with_status("", StatusCode::CONFLICT))
        }
    }
    match sqlx::query("insert into connected_apps(user_id, sub, client_id, app_host) values ($1, $2, $3, $4)")
            .bind(user_id)
            .bind(&appinfo.sub)
            .bind(&appinfo.client_id)
            .bind(&app_host)
            .execute(&db_pool).await {
        Ok(_) => {
            Ok(warp::reply::with_status("", StatusCode::CREATED))
        },
        Err(error) => {
            log::error!("Error storing association between {} and {}/{}/{}: {}",
                user_id, appinfo.sub, appinfo.client_id, app_host, error);
            Ok(warp::reply::with_status("", StatusCode::CONFLICT))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test covers both hashing and verification because hashing cannot be tested without
    // using the external crate since the salt is non-deterministic (and this solution is both
    // simpler and more secure than introducing a option for deterministic salts)
    #[test]
    fn test_hash_and_verify_password() {
        let hashed_password = hash_password("password")
            .expect("Password hashing failed");
        let password_is_ok = verify_password("password", &hashed_password)
            .expect("Password verification failed");
        assert_eq!(password_is_ok, true);
    }

    #[test]
    fn test_hash_and_verify_password_not_ok() {
        let hashed_password = hash_password("password2")
            .expect("Password hashing failed");
        let password_is_ok = verify_password("password", &hashed_password)
            .expect("Password verification failed");
        assert_eq!(password_is_ok, false);
    }

    #[test]
    fn test_validate_password_chars() {
        assert_eq!(validate_password_chars("Passw*ord123"), true);
    }

    #[test]
    fn test_validate_password_chars_fails() {
        assert_eq!(validate_password_chars("Pass"), false);
    }

    #[test]
    fn test_validate_username_chars() {
        assert_eq!(validate_username_chars("username"), true);
    }

    #[test]
    fn test_validate_username_chars_fails() {
        assert_eq!(validate_username_chars(" username; drop table users"),
            false);
    }

    #[test]
    fn test_create_jwt() {
        let secret = b"secret";
        let jwt = create_jwt("username", secret, 60).expect("Unable to create jwt");
        let decoded = jsonwebtoken::decode::<Claims>(&jwt,
            &jsonwebtoken::DecodingKey::from_secret(secret),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512))
            .expect("Unable to decode jwt");
        assert_eq!("username", decoded.claims.sub);
    }

    #[test]
    fn test_generate_random_string() {
        let generated_string = String::from_utf8(generate_random_string())
            .expect("Error parsing generated string");
        let entropy = zxcvbn::zxcvbn(&generated_string, &[])
            .expect("Error computing string entropy");
        assert_eq!(entropy.score(), 4);
    }
}