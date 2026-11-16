use atomic_delegate_macros::atomic_delegates;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct BrpClient {
    pub(crate) url: String,
    pub(crate) http: reqwest::Client,
}

impl BrpClient {
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            url: format!("http://127.0.0.1:{port}/"),
            http: reqwest::Client::new(),
        }
    }
}

#[atomic_delegates]
impl BrpClient {
    pub async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, String> {
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::BrpClient;

    #[tokio::test]
    async fn test_usage() {
        let client = BrpClient::new(1);
        let result = client.call::<Value>("rpc.discover", Value::Null).await;
        assert!(result.is_err());
    }
}
