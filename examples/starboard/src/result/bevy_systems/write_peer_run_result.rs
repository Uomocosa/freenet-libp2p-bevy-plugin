use bevy::prelude::*;

use crate::result::ResultJsonPath;
use examples_shared::e2e_report::peer_run_result::PeerRunResult;

pub fn write_peer_run_result(
    mut exits: MessageReader<AppExit>,
    time: Res<Time>,
    mut result: ResMut<PeerRunResult>,
    path: Res<ResultJsonPath>,
    mut last_snapshot_secs: Local<Option<f64>>,
) {
    let path = path.into_inner();
    let time = time.into_inner();
    if let Some(exit) = exits.read().next() {
        result.exit_code = match exit {
            AppExit::Success => 0,
            AppExit::Error(code) => code.get(),
        };
    } else {
        let every = ResultJsonPath::snapshot_every().as_secs_f64();
        let snapshot_due =
            last_snapshot_secs.is_none_or(|last| time.elapsed_secs_f64() - last >= every);
        if !snapshot_due || !ResultJsonPath::has_room_evidence(&result) {
            return;
        }
        *last_snapshot_secs = Some(time.elapsed_secs_f64());
    }
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

    use super::write_peer_run_result;
    use crate::result::ResultJsonPath;
    use examples_shared::e2e_report::peer_run_result::PeerRunResult;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("p1").join("result.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(PeerRunResult {
            username: "p1".to_string(),
            ..PeerRunResult::default()
        });
        app.insert_resource(ResultJsonPath(path.clone()));
        app.add_systems(Last, write_peer_run_result);
        app.world_mut().write_message(AppExit::Success);
        app.update();
        let bytes = std::fs::read(&path).expect("result.json");
        let back: PeerRunResult = serde_json::from_slice(&bytes).expect("decode");
        assert_eq!(back.username, "p1");
        assert_eq!(back.exit_code, 0);
    }

    /// A peer whose script ends in `stay` never sees `AppExit`, so the
    /// periodic snapshot is its only `result.json` (S4a holders).
    #[test]
    fn staying_peers_snapshot_result_without_exit() {
        use freenet_libp2p_bevy_plugin::identity;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("p1").join("result.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        let mut recorded = PeerRunResult {
            username: "p1".to_string(),
            ..PeerRunResult::default()
        };
        recorded.members_seen.insert(identity::PeerId::from("p1"));
        app.insert_resource(recorded);
        app.insert_resource(ResultJsonPath(path.clone()));
        app.add_systems(Last, write_peer_run_result);
        app.update();
        let bytes = std::fs::read(&path).expect("staying peers leave a result.json");
        let back: PeerRunResult = serde_json::from_slice(&bytes).expect("decode");
        assert_eq!(back.username, "p1");
    }

    /// A watcher that never joined a room leaves no `result.json`, exactly as
    /// before snapshots existed, so `may_crash` keeps excusing it.
    #[test]
    fn watchers_without_room_evidence_leave_no_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("p3").join("result.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(PeerRunResult {
            username: "p3".to_string(),
            ..PeerRunResult::default()
        });
        app.insert_resource(ResultJsonPath(path.clone()));
        app.add_systems(Last, write_peer_run_result);
        app.update();
        assert!(
            std::fs::read(&path).is_err(),
            "watchers without room evidence leave no result.json"
        );
    }
}
