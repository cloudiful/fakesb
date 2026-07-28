mod app;
mod domain;
mod http;
mod migrations;
mod openapi;
mod repositories;
mod service;
mod template;

use crate::app::bootstrap;
use utoipa::OpenApi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("export-openapi") {
        println!(
            "{}",
            serde_json::to_string_pretty(&openapi::ApiDoc::openapi())
                .map_err(|err| std::io::Error::other(err.to_string()))?
        );
        return Ok(());
    }

    let app = bootstrap("fakesb").await?;
    app.run().await
}
