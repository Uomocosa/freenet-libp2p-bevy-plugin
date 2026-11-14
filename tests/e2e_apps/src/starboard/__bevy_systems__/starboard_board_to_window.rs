use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use starboard::starboard::render::CanvasView;

pub fn starboard_board_to_window(
    In(params): In<Option<Value>>,
    view: Res<CanvasView>,
) -> BrpResult {
    let view = view.into_inner();
    let cells: Vec<(f32, f32)> =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    let points: Vec<Vec2> = cells
        .into_iter()
        .map(|(x, y)| {
            Vec2::new(
                (x + 0.5).mul_add(view.scale, view.offset.x),
                (y + 0.5).mul_add(view.scale, view.offset.y),
            )
        })
        .collect();
    serde_json::to_value(points).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::json;

    use starboard::starboard::render::CanvasView;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(CanvasView {
            scale: 2.0,
            offset: Vec2::new(10.0, 20.0),
        });
        let system = app
            .world_mut()
            .register_system(super::starboard_board_to_window);
        let result = app
            .world_mut()
            .run_system_with(system, Some(json!([[0.0, 0.0], [3.0, 1.0]])));
        let Ok(Ok(value)) = result else {
            panic!("board_to_window did not run: {result:?}");
        };
        let points: Vec<Vec2> = serde_json::from_value(value).expect("decode");
        assert_eq!(points, vec![Vec2::new(11.0, 21.0), Vec2::new(17.0, 23.0)]);
    }
}
