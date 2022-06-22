// use anyhow::Result;
// use cadre::template::Template;
// use serde_json::Value;

// #[tokio::test]
// async fn parse_test() -> Result<()> {
//     let observed = Value::from("{}");
//     let expected = Value::from("{}");
//     let template = Template::new(observed).await?;

//     assert_eq!(template.parse().await?, Value::from(expected));
//     Ok(())
// }
