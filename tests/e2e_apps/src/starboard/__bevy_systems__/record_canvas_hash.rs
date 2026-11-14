use bevy::prelude::*;

use crate::starboard::CanvasHashWatch;
use starboard::starboard::canvas::{Canvas, hash_canvas};

pub fn record_canvas_hash(canvas: Res<Canvas>, time: Res<Time>, watch: ResMut<CanvasHashWatch>) {
    if !canvas.is_changed() {
        return;
    }
    let hash = hash_canvas(canvas.into_inner());
    let now = time.into_inner().elapsed();
    let watch = watch.into_inner();
    if hash != watch.hash {
        watch.hash = hash;
        watch.changed_at = now;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::starboard::CanvasHashWatch;
    use starboard::starboard::canvas::{Canvas, CellPos, Layer, hash_canvas, write_cell};
    use starboard::starboard::identity::PaletteColor;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Canvas>()
            .init_resource::<CanvasHashWatch>()
            .add_systems(Update, super::record_canvas_hash);
        app.update();
        let empty = hash_canvas(&Canvas::default());
        assert_eq!(app.world().resource::<CanvasHashWatch>().hash, empty);

        let mut canvas = app.world_mut().resource_mut::<Canvas>();
        let _ = write_cell(
            &mut canvas,
            CellPos { x: 4, y: 4 },
            Layer {
                rank: 1,
                author: 1,
                color: Some(PaletteColor(0)),
                mask: starboard::starboard::canvas::FULL_MASK,
            },
        );
        app.update();
        let watch = *app.world().resource::<CanvasHashWatch>();
        assert_ne!(watch.hash, empty);
        assert_eq!(watch.changed_at, app.world().resource::<Time>().elapsed());
    }
}
