use std::io::BufRead;

use rand::{SeedableRng,Rng};
use rand::rngs::StdRng;
use sqlx::PgPool;

// This function creates a new database with a random name in order to run each test in a
// separate environment.
pub async fn create_db() -> PgPool {
    let connection_string = std::env::var("TEST_DB").expect("`TEST_DB` variable not set");
    let root_conn = PgPool::connect(&format!("{}/postgres", connection_string)).await
        .expect("Unable to open postgres database");
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = StdRng::from_entropy();
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

pub async fn execute_sql_from_file(path: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let file = std::fs::File::open(path).expect(&format!("Error reading {}", path));
    let lines: Vec<String> = std::io::BufReader::new(file).lines().map(|l| l.expect("Error reading line")).collect();
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("");
    for line in lines {
        builder.push(&line);
        if line.trim_end().ends_with(";") {
            let q = builder.build();
            q.execute(&mut tx).await?;
            builder.reset();
        }
    }
    tx.commit().await?;
    Ok(())
}
