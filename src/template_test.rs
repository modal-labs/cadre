use anyhow::Result;
use cadre::Template;
use serde_json::Value;


#[tokio::test]
async fn parse_test() -> Result<()> {
    let value = Value::from_str("{}");
    let template = Template::new(value);

    assert_eq!(template.parse().await?, value);
    Ok(())
}
