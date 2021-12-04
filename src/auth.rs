use argon2::password_hash::{PasswordHash,SaltString};
use argon2::{Argon2,PasswordHasher,PasswordVerifier};
use sqlx::sqlite::SqlitePool;
use serde::Deserialize;
use serde::Serialize;
use warp::http::{StatusCode,Response};

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
struct AuthConfigError {
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

impl From<std::env::VarError> for AuthConfigError {
    fn from(error: std::env::VarError) -> Self {
        AuthConfigError {message: error.to_string()}
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(error: argon2::password_hash::Error) -> Self {
        AuthError {message: error.to_string()}
    }
}

#[derive(Deserialize,Debug)]
pub struct User {
    pub username: String,
    pub password: String,
}

pub fn decode_basic_auth_header(encoded_auth: &str) -> Result<User, AuthError> {
    base64::decode_config(encoded_auth, base64::STANDARD)
        .map_or_else(|error| {
            Err(AuthError {message: format!("Unable to decode provided header {}: {}", encoded_auth, error)})
        }, |decoded_bytes| {
            match String::from_utf8(decoded_bytes) {
                Ok(decoded_bytes) => Ok(decoded_bytes),
                Err(error) => Err(AuthError {message: format!("Unable to decode provided header {}: {}", encoded_auth, error)})
            }
        })
        .and_then(|credentials|{
            let parts = credentials.splitn(2, ':').collect::<Vec<&str>>();
            if parts.len() != 2 {
                Err(AuthError {message: format!("Unable parse decoded credentials {}", credentials)})
            } else {
                Ok(User {username: parts[0].to_string(), password: parts[1].to_string()})
            }
        })
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

pub async fn register_handler(db_pool: SqlitePool, body: User) ->
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
            eprintln!("Error hashing password for user {}: {}", &body.username, error.to_string());
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid password".to_string())
            )
        }
    };
    match sqlx::query("INSERT INTO users(username, password_hash) VALUES($1, $2)")
            .bind(&body.username)
            .bind(&hashed_password)
            .execute(&db_pool).await {
        Ok(_) => return Ok(Response::builder().status(StatusCode::CREATED).body("".to_string())),
        Err(error) => {
            eprintln!("Error writing user entry for user {}: {}", &body.username, error.to_string());
            return Ok(Response::builder()
                .status(StatusCode::CONFLICT)
                .body("This email is already in use".to_string())
            );
        }
    }
}

fn verify_user_with_password(provided_pass: &str, optional_hashed_pass: &Option<(i64, String,)>) -> Result<Option<i64>, AuthError> {
    let fallback_password = "$argon2id$v=19$m=4096,t=3,p=1$ewSM8Hmctto5QHVv27S1cA$o6GeMd3PriFhi2CalkBmG1cV/AMi+ry0r/6fjmeSaFQ";
    match optional_hashed_pass {
        Some((user_id, password_hash,)) => {
            if verify_password(provided_pass, &password_hash)? {
                Ok(Some(user_id.to_owned()))
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

pub async fn verify_password_from_database(db_pool: &SqlitePool, username: &str, password: &str)
        -> Result<Option<i64>, AuthError> {
    sqlx::query_as::<_, (i64, String,)>("SELECT id, password_hash FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(db_pool).await
            .map_or_else(|error| {
                Err(AuthError {message: format!("Error getting password from database for user {}: {}", username, error)})
            }, |optional_hashed_pass| {
                Ok(verify_user_with_password(password, &optional_hashed_pass)?)
            })
}

pub async fn verify_user_handler(db_pool: SqlitePool, body: User) ->
        Result<impl warp::Reply, warp::Rejection> {
    verify_password_from_database(&db_pool, &body.username, &body.password)
        .await
        .map_or_else(|error| {
                eprintln!("{}", error);
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("".to_string()))
                }, |verified_user_id| {
                        if verified_user_id.is_some() {
                            match create_jwt(&body.username, 60) {
                                Ok(jwt) => Ok(Response::builder()
                                    .status(StatusCode::OK)
                                    .body(jwt)),
                                Err(error) => {
                                    eprintln!("Error when creating jwt for user {}: {}", &body.username, error);
                                    Ok(Response::builder()
                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                        .body("".to_string()))
                                }
                            }
                        } else {
                            return Ok(Response::builder()
                                .status(StatusCode::UNAUTHORIZED)
                                .body("Password doesn't match".to_string()))
                        }
                })
}

// The naming of these fields is significant: https://github.com/Keats/jsonwebtoken#validation
#[derive(Debug,Deserialize,Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn create_jwt(username: &str, expiration_in_seconds: i64) -> Result<String, AuthConfigError> {
    let experiation = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expiration_in_seconds))
            .ok_or_else(|| AuthConfigError {
                message: format!("Error adding {} seconds to the current time", expiration_in_seconds)
            })?
        .timestamp() as usize;
    let claims = Claims {sub: username.to_string(), exp: experiation};
    let secret = std::env::var("JWT_SECRET_KEY")?;
    Ok(jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512),
        &claims, &jsonwebtoken::EncodingKey::from_secret(secret.as_ref())
    )?)
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    Ok(Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string())
}

fn verify_password(password: &str, hashed_password: &str) -> Result<bool, argon2::password_hash::Error> {
    let hash = PasswordHash::new(&hashed_password)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &hash).is_ok())
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
        let mut env_set = false;
        if let None = std::env::var_os("JWT_SECRET_KEY") {
            std::env::set_var("JWT_SECRET_KEY", "secret");
            env_set = true;
        }
        let secret = std::env::var("JWT_SECRET_KEY").expect("JWT secret not set");
        let jwt = create_jwt("username", 60).expect("Unable to create jwt");
        if env_set {
            std::env::remove_var("JWT_SECRET_KEY");
        }
        let decoded = jsonwebtoken::decode::<Claims>(&jwt,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512))
            .expect("Unable to decode jwt");
        assert_eq!("username", decoded.claims.sub);
    }
}
