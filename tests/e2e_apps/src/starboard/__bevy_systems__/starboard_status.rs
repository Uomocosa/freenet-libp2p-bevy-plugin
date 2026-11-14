use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::starboard::{CanvasHashWatch, CanvasStatus};

pub fn starboard_status(
    In(_params): In<Option<Value>>,
    canvas: Res<starboard::starboard::canvas::Canvas>,
    watch: Res<CanvasHashWatch>,
    time: Res<Time>,
) -> BrpResult {
    let canvas = canvas.into_inner();
    let watch = watch.into_inner();
    let time = time.into_inner();
    let status = CanvasStatus {
        hash: watch.hash,
        painted_tiles: canvas.tiles.iter().filter(|tile| tile.is_some()).count(),
        stable_for_ms: u64::try_from(time.elapsed().saturating_sub(watch.changed_at).as_millis())
            .unwrap_or(u64::MAX),
    };
    serde_json::to_value(status).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use crate::starboard::{CanvasHashWatch, CanvasStatus};
    use starboard::starboard::canvas::Canvas;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Canvas>()
            .insert_resource(CanvasHashWatch {
                hash: 9,
                changed_at: Duration::from_secs(1),
            });
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(1500));
        let system = app.world_mut().register_system(super::starboard_status);
        let result = app.world_mut().run_system_with(system, None);
        let Ok(Ok(value)) = result else {
            panic!("status did not run: {result:?}");
        };
        let status: CanvasStatus = serde_json::from_value(value).expect("decode");
        assert_eq!(status.hash, 9);
        assert_eq!(status.painted_tiles, 0);
        assert_eq!(status.stable_for_ms, 500);
    }
}
