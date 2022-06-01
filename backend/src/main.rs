use article_server_rs::{setup_args,start_server,migrate_db,ServerArgs};

use std::io::Write;

use sqlx::PgPool;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .format(|buf, record| {
            let message = serde_json::json!({
                "level": record.level().to_string(),
                "message": record.args().as_str().map_or_else(|| {record.args().to_string()}, |s| s.to_string()),
                "target": record.target().to_string(),
            });
            writeln!(buf, "{}", message.to_string())
        })
        .filter(None, log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();
    let args = setup_args();
    let port = args.value_of("port").expect("Unable to get port argument")
        .parse::<u16>().expect("Unable to parse port argument");
    let host = args.value_of("host").expect("Unable to get host argument");
    let service_address_str = format!("{}:{}", host, port);
    let db_url = args.value_of("database-path")
        .expect("Unable to get database-path argument");
    let pool = PgPool::connect(db_url).await.expect("Unable to get database connection pool");
    migrate_db(&pool).await.expect("Unable to migrate database");
    let server_args = ServerArgs {
        pool,
        addr: service_address_str.parse().expect(
            &format!("Unable to parse {} as a socket address", service_address_str)),
    };
    start_server(server_args).await;
}
