use crate::testing;

pub async fn post_peer_descriptions(
    board: &testing::harness::GitHubCheckpointBoard,
    name: &str,
    descriptions: &[String],
) -> Result<(), String> {
    for (index, description) in descriptions.iter().enumerate() {
        let context =
            testing::harness::GitHubCheckpointBoard::peer_context(&board.prefix, name, index);
        board.post_status(&context, description).await?;
    }
    Ok(())
}

// no test_usage necessary
