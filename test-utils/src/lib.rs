use rand::Rng;
use sqlx::PgPool;

// This function creates a new database with a random name in order to run each test in a
// separate environment.
pub async fn create_db() -> PgPool {
    let connection_string = std::env::var("TEST_DB").expect("`TEST_DB` variable not set");
    let root_conn = PgPool::connect(&format!("{}/postgres", connection_string)).await
        .expect("Unable to open postgres database");
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    let db_name = (0..10).map(|_| {
        let idx = rng.gen_range(0..charset.len());
        charset[idx]
    }).collect();
    let db_name = String::from_utf8(db_name).expect("Unable to generate random db name");
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&root_conn)
        .await
        .expect("Unable to create new database");
    let conn = PgPool::connect(&format!("{}/{}", connection_string, db_name)).await
        .expect("Unable to open created database");
    conn
}
