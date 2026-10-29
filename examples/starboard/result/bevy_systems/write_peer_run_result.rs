use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::e2e_report;

use crate::result::ResultJsonPath;

pub fn write_peer_run_result(
    mut exits: MessageReader<AppExit>,
    mut result: ResMut<e2e_report::PeerRunResult>,
    path: Res<ResultJsonPath>,
) {
    let path = path.into_inner();
    let Some(exit) = exits.read().next() else {
        return;
    };
    result.exit_code = match exit {
        AppExit::Success => 0,
        AppExit::Error(code) => code.get(),
    };
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        tracing::error!("result.json parent dir failed: {error}");
        return;
    }
    match serde_json::to_vec_pretty(&*result) {
        Ok(bytes) => {
            if let Err(error) = std::fs::write(path.as_path(), bytes) {
                tracing::error!("result.json write failed: {error}");
            }
        }
        Err(error) => {
            tracing::error!("result.json encode failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::e2e_report;

    use super::write_peer_run_result;
    use crate::result::ResultJsonPath;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("p1").join("result.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(e2e_report::PeerRunResult {
            username: "p1".to_string(),
            ..e2e_report::PeerRunResult::default()
        });
        app.insert_resource(ResultJsonPath(path.clone()));
        app.add_systems(Last, write_peer_run_result);
        app.world_mut().write_message(AppExit::Success);
        app.update();
        let bytes = std::fs::read(&path).expect("result.json");
        let back: e2e_report::PeerRunResult = serde_json::from_slice(&bytes).expect("decode");
        assert_eq!(back.username, "p1");
        assert_eq!(back.exit_code, 0);
    }
}
