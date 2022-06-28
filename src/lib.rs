//! Cadre is a simple, self-hosted, high-performance remote configuration
//! service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod client;
pub mod secrets;
pub mod server;
pub mod storage;
pub mod template;

pub use crate::cli::Args;
pub use crate::client::CadreClient;

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::json;

    use crate::{secrets::Secrets, template::populate_template};

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
}
