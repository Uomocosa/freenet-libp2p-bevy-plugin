use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::testing;

pub async fn call<T: DeserializeOwned>(
    client: &testing::harness::BrpClient,
    method: &str,
    params: Value,
) -> Result<T, String> {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    let response: Value = client
        .http
        .post(&client.url)
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

// no test_usage necessary
