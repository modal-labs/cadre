use anyhow::bail;
use anyhow::Result;
use serde_json::Value;

pub struct Template {
    value: Value,
}

impl Template {
    /// Creates new template based on serde JSON Value.
    pub async fn new(value: Value) -> Result<Self> {
        Ok(Self { value: value })
    }

    /// Parses JSON map based on template requirements.
    pub async fn parse(&self) -> Result<()> {
        if self.value.is_array() {
            bail!("config objects cannot be arrays")
        } else {
            let map = self.value.as_object().unwrap();
            for (key, value) in map.iter() {
                println!("{}, {}", key, value);
            }
            Ok(())
        }
    }
}

/// Gets secret from the AWS secret manager.
fn get_aws_secret(secret_name: String) -> Result<()> {
    Ok(())
}
