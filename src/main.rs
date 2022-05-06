use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let app = cadre::server().await?;
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
