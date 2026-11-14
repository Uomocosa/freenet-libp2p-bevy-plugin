use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use starboard::starboard::canvas::{CellPos, read_cell};

pub fn starboard_cell_painted(
    In(params): In<Option<Value>>,
    canvas: Res<starboard::starboard::canvas::Canvas>,
) -> BrpResult {
    let (x, y): (u16, u16) =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    let painted = read_cell(canvas.into_inner(), CellPos { x, y })
        .top
        .color
        .is_some();
    serde_json::to_value(painted).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::{RemoteMethods, RemotePlugin};
    use serde_json::{Value, json};

    use starboard::starboard::canvas::{Canvas, CellPos, Layer, write_cell};
    use starboard::starboard::identity::PaletteColor;

    fn painted_at(canvas: Canvas, params: Value) -> Option<bool> {
        let mut app = App::new();
        app.insert_resource(canvas);
        let system = app
            .world_mut()
            .register_system(super::starboard_cell_painted);
        let result = app.world_mut().run_system_with(system, Some(params));
        result
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok())
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            RemotePlugin::default()
                .with_method_main("starboard/cell_painted", super::starboard_cell_painted),
        );
        let methods = app.world().resource::<RemoteMethods>();
        assert!(methods.get("starboard/cell_painted").is_some());
    }

    #[test]
    fn an_unpainted_cell_reads_false_and_a_painted_cell_reads_true() {
        let mut canvas = Canvas::default();
        let layer = Layer {
            rank: 1,
            author: 1,
            color: Some(PaletteColor(1)),
            mask: u16::MAX,
        };
        assert!(write_cell(&mut canvas, CellPos { x: 200, y: 300 }, layer));
        assert_eq!(
            painted_at(Canvas::default(), json!([200, 300])),
            Some(false)
        );
        assert_eq!(painted_at(canvas.clone(), json!([200, 300])), Some(true));
        assert_eq!(painted_at(canvas, json!([201, 300])), Some(false));
    }
}
