use serde::de::DeserializeOwned;
use serde_json::{Value, json};

pub struct BrpClient {
    url: String,
    http: reqwest::Client,
}

impl BrpClient {
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            url: format!("http://127.0.0.1:{port}/"),
            http: reqwest::Client::new(),
        }
    }

    /// # Errors
    /// Returns an error if the request fails, the server answers with a JSON-RPC error,
    /// or the result does not decode as `T`.
    pub async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, String> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        let response: Value = self
            .http
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("{method}: {e}"))?
            .json()
            .await
            .map_err(|e| format!("{method}: reading response: {e}"))?;
        if let Some(error) = response.get("error") {
            return Err(format!("{method}: {error}"));
        }
        let result = response.get("result").cloned().unwrap_or(Value::Null);
        serde_json::from_value(result).map_err(|e| format!("{method}: decoding result: {e}"))
    }
}
