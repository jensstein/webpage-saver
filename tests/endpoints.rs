use std::net::{SocketAddr,TcpListener};

use test_utils::create_db;

use article_server_rs::{migrate_db,ServerArgs,start_server};

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
    let addr = get_address();
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
        setup_server(addr).await.await;
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}:{}/api/status", addr.ip(), addr.port()))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
}

async fn setup_server(addr: SocketAddr) -> impl core::future::Future {
    let pool = create_db().await;
    migrate_db(&pool).await.expect("Unable to migrate database");
    let server_args = ServerArgs {
        pool,
        addr,
    };
    start_server(server_args)
}
