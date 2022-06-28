use anyhow::Result;
use cadre::storage::default_aws_config;
use cadre::template::Template;
use serde_json::{json, Value};

#[tokio::test]
async fn parse_test() -> Result<()> {
    let config = default_aws_config().await?;
    let json = json!({"name": "test"});
    let observed = json.clone();
    let expected = json.clone();
    let mut template = Template::new(&config, observed).await?;

    assert_eq!(template.parse().await?, expected);
    Ok(())
}

#[tokio::test]
async fn parse_test_fail() -> Result<()> {
    let config = default_aws_config().await?;
    let observed = Value::from("[]");
    let mut template = Template::new(&config, observed).await?;

    let value = template.parse().await;
    assert!(value.is_err());
    Ok(())
}
