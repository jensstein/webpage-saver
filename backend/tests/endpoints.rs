use std::net::{SocketAddr,TcpListener};

use sqlx::PgPool;

use test_utils::{create_db, execute_sql_from_file, init_logging};

use article_server_rs::{migrate_db,ServerArgs,start_server,
    auth::{create_jwt,Role},
    webpages::ShowWebpageResponse};

struct TestResources {
    addr: SocketAddr,
    pool: PgPool,
    jwt: String,
    admin_jwt: String,
}

// Get a random port. The TcpListener has to go out of scope to be closed and thereby release its
// bind on the address. Otherwise the server cannot bind the address since it will already be
// bound.
fn get_address() -> SocketAddr {
    // binding to port 0 gives you a random port on linux.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Unable to bind tcp listener");
    listener.local_addr().expect("Unable to get socket address")
}

#[tokio::test]
async fn test_status() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}:{}/api/status",
            test_resources.addr.ip(), test_resources.addr.port()))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
}

#[tokio::test]
async fn test_get_webpage() {
    let test_resources = start_test_server().await;
    execute_sql_from_file("tests/data/insert-webpage.sql", &test_resources.pool)
        .await.expect("Unable to insert webpages");
    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}:{}/api/webpage/1",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let webpage: ShowWebpageResponse = serde_json::from_str(
        &response.text().await.expect("Unable to get text from response"))
        .expect("Unable to parse response as webpage response");
    assert_eq!(webpage, ShowWebpageResponse::new(
        "title".into(),
        Some("image_url".into()),
        "text".into(),
    ));
}

#[tokio::test]
async fn test_delete_webpage() {
    let test_resources = start_test_server().await;
    execute_sql_from_file("tests/data/insert-webpage.sql", &test_resources.pool).await.expect("Unable to insert webpages");
    let results = sqlx::query_as::<_, (String, String)>("SELECT url, title FROM webpages WHERE id = 1")
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for webpage prior to deleting");
    assert_eq!(results.is_some(), true);
    let client = reqwest::Client::new();
    let response = client.delete(format!("http://{}:{}/api/webpage/1",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let results = sqlx::query_as::<_, (String, String)>("SELECT url, title FROM webpages WHERE id = 1")
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for webpage after deleting");
    assert_eq!(results.is_some(), false);
}

#[tokio::test]
async fn test_register_user() {
    let test_resources = start_test_server().await;
    let users_pre = sqlx::query_as::<_, (i64,)>("select count(id) from users")
        .fetch_one(&test_resources.pool).await.expect("Unable to get user count before registering");
    assert_eq!(users_pre.0, 2);
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/register",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"username": "new-user", "password": "Password123"}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.admin_jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let users_post = sqlx::query_as::<_, (String, Vec<Role>)>("select username, roles from users where username = 'new-user'")
        .fetch_one(&test_resources.pool).await.expect("Unable to get user count before registering");
    assert_eq!(users_post.0, "new-user");
    assert_eq!(users_post.1.len(), 1);
    assert_eq!(users_post.1[0], Role::User);
}

#[tokio::test]
async fn test_register_user_not_allowed_for_non_admins() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/register",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"username": "new-user", "password": "Password123"}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::UNAUTHORIZED);
    let new_user = sqlx::query_as::<_, (String, Vec<Role>)>("select username, roles from users where username = 'new-user'")
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for new user");
    assert!(new_user.is_none());
}

#[tokio::test]
async fn test_associate_app_to_user() {
    let test_resources = start_test_server().await;
    let apps_pre = sqlx::query_as::<_, (i64,)>("select count(id) from connected_apps")
        .fetch_one(&test_resources.pool).await.expect("Unable to get app count before registering");
    assert_eq!(apps_pre.0, 0);
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/associate-app-to-user",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"sub": "sub", "client_id": "client_id",
            "app_host": "app_host"}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::CREATED);
    let apps_post = sqlx::query_as::<_, (i64,String,String,String)>(
            "select user_id, sub, client_id, app_host from connected_apps where user_id = 1")
        .fetch_optional(&test_resources.pool).await.expect(
            "Unable to get app count before registering");
    assert_eq!(apps_post.is_some(), true);
    if let Some(apps_post) = apps_post {
        let (user_id, sub, client_id, app_host) = apps_post;
        assert_eq!(user_id, 1);
        assert_eq!(sub, "sub");
        assert_eq!(client_id, "client_id");
        assert_eq!(app_host, "app_host");
    }
}

#[tokio::test]
async fn test_associate_app_to_user_existing_app() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();

    sqlx::query("insert into connected_apps(user_id, sub, client_id, app_host) values (1, 'sub', 'client_id', 'app_host')")
        .execute(&test_resources.pool).await
        .expect("Unable to insert app into database");

    let response = client.post(format!("http://{}:{}/api/associate-app-to-user",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"sub": "sub", "client_id": "client_id",
            "app_host": "app_host"}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::CREATED);
    let apps_post = sqlx::query_as::<_, (i64,String,String,String)>(
            "select user_id, sub, client_id, app_host from connected_apps where user_id = 1 order by id desc")
        .fetch_one(&test_resources.pool).await.expect(
            "Unable to get app count before registering");
    let (user_id, sub, client_id, app_host) = apps_post;
    assert_eq!(user_id, 1);
    assert_eq!(sub, "sub");
    assert_eq!(client_id, "client_id");
    // The app host must be something other than "app_host" because that values already exists.
    assert!(app_host != "app_host");
}

#[tokio::test]
async fn test_associate_app_to_user_malformed_json() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/associate-app-to-user",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"hur": 123}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::BAD_REQUEST);
}

async fn start_test_server() -> TestResources {
    init_logging();
    let addr = get_address();
    let pool = create_db().await;
    let cloned_pool = pool.clone();
    migrate_db(&pool).await.expect("Unable to migrate database");
    execute_sql_from_file("tests/data/auth.sql", &pool).await.expect("Unable to insert auth data");
    let (user, secret) = sqlx::query_as::<_, (String, String)>("SELECT users.username, jwt_secrets.secret FROM jwt_secrets JOIN users on users.id = jwt_secrets.user_id where users.id = 1").fetch_one(&pool).await.expect("Unable to fetch jwt secret");
    let jwt = create_jwt(&user, secret.as_ref(), 60 * 60 * 24).expect("Error creating jwt");
    let (admin_user, admin_secret) = sqlx::query_as::<_, (String, String)>("SELECT users.username, jwt_secrets.secret FROM jwt_secrets JOIN users on users.id = jwt_secrets.user_id where users.id = 2").fetch_one(&pool).await.expect("Unable to fetch admin jwt secret");
    let admin_jwt = create_jwt(&admin_user, admin_secret.as_ref(), 60 * 60 * 24).expect("Error creating admin jwt");
    let _ = tokio::spawn(async move {
        // Starting a task with a server started by warp::Server::run is possibly impossible to do
        // at the moment. I get the error
        // 28 |         rt.spawn(setup_server(addr));
        //    |            ^^^^^ implementation of `warp::reply::Reply` is not general enough
        //    |
        //    = note: `&'0 str` must implement `warp::reply::Reply`, for any lifetime `'0`...
        //    = note: ...but `warp::reply::Reply` is actually implemented for the type `&'static str`
        // I could get it to work by using Server::bind instead but that then creates an
        // intermediate Future and that's why I have the double await here.
        setup_server(addr, cloned_pool).await.await;
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    TestResources {
        addr,
        pool,
        jwt,
        admin_jwt,
    }
}

async fn setup_server(addr: SocketAddr, pool: PgPool) -> impl core::future::Future {
    let server_args = ServerArgs {
        pool,
        addr,
    };
    start_server(server_args)
}
