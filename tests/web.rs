//! Integration tests for the web server.

use std::net::TcpListener;

use anyhow::Result;
use cadre::server::{
    resolver::{EchoJson, ResolverChain},
    server,
    state::State,
    storage::Storage,
};
use cadre::CadreClient;
use serde_json::json;
use tokio::task::JoinHandle;

async fn spawn_test_server() -> Result<(CadreClient, JoinHandle<()>)> {
    let mut chain = ResolverChain::new();
    chain.add(EchoJson);
    let storage = Storage::Memory(Default::default());
    let state = State::new(chain, storage, Some("default"));

    let app = server(state);

    let listener = TcpListener::bind("localhost:0")?;
    let client = CadreClient::new(&format!("http://{}", listener.local_addr()?));

    let handle = tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok((client, handle))
}

#[tokio::test]
async fn simple_operations() -> Result<()> {
    let (client, _handle) = spawn_test_server().await?;

    assert!(client.list_configs().await?.is_empty());
    assert!(client.load_config("hello").await.is_err());

    client
        .write_template("hello", &json!({ "foo": "bar" }))
        .await?;

    assert!(client.load_config("hello").await.is_err());

    client.write_template("default", &json!({})).await?;
    assert_eq!(client.load_config("hello").await?, json!({ "foo": "bar" }));
    assert_eq!(
        client.list_configs().await?,
        vec![String::from("default"), String::from("hello")]
    );

    Ok(())
}

#[tokio::test]
async fn override_resolvers() -> Result<()> {
    let (client, _handle) = spawn_test_server().await?;

    client
        .write_template("default", &json!({ "foo": "bar" }))
        .await?;
    client
        .write_template("hello", &json!({ "*foo": "echo:\"banana\"" }))
        .await?;

    assert_eq!(
        client.load_config("hello").await?,
        json!({ "foo": "banana" })
    );

    Ok(())
}
