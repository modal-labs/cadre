use anyhow::Result;
use cadre::{secrets::Secrets, template::populate_template};
use serde_json::json;

#[tokio::test]
async fn populate_basic() -> Result<()> {
    let secrets = Secrets::new_test();
    let mut template = json!({"name": "test"});
    let expected = template.clone();

    populate_template(&mut template, &secrets).await?;
    assert_eq!(template, expected);
    Ok(())
}

#[tokio::test]
async fn populate_fail() -> Result<()> {
    let secrets = Secrets::new_test();
    let mut template = json!({"*mykey": "NotAValidTemplateLiteral!#@"});
    let result = populate_template(&mut template, &secrets).await;
    assert!(result.is_err());
    Ok(())
}
