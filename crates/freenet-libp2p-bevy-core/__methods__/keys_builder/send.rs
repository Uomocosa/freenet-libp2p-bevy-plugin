use serde_json::Value;

use crate::testing;

pub async fn send(builder: testing::harness::KeysBuilder<'_>) -> Result<(), String> {
    let params =
        serde_json::to_value(&builder.steps).map_err(|error| format!("keys steps: {error}"))?;
    builder
        .client
        .call::<Value>("e2e/keys", params)
        .await
        .map_err(|error| format!("e2e/keys: {error}"))?;
    testing::harness::Polling::from(builder.deadlines.action())
        .until(
            "input delivered",
            async || {
                builder
                    .client
                    .call::<bool>("e2e/input_idle", Value::Null)
                    .await
            },
            |idle| *idle,
        )
        .await
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::send;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let client = testing::harness::BrpClient::new(1);
        let builder = testing::harness::KeysBuilder::new(&client, testing::harness::Deadlines(1.0))
            .tap(testing::Key::Enter);
        assert!(send(builder).await.is_err());
    }
}
