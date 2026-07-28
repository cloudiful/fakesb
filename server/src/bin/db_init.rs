use std::io;

#[path = "../migrations.rs"]
mod migrations;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename("../.env");

    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "DATABASE_URL must be set before running db_init",
        )
    })?;

    if database_url.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "DATABASE_URL must not be empty",
        )
        .into());
    }

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;
    migrations::MIGRATOR.run(&pool).await?;
    pool.close().await;

    println!("database migrations applied");
    Ok(())
}
