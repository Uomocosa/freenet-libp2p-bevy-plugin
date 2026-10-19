use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::args::ScriptArg;
use crate::starboard;

const SCRIPT_START_SECONDS: f32 = 3.0;

#[derive(Resource, Debug, Clone, Copy)]
pub struct StrokeScript {
    pub action: ScriptArg,
    pub drawn: u32,
    pub waited_secs: f32,
}

impl StrokeScript {
    #[must_use]
    pub const fn new(action: ScriptArg) -> Self {
        Self {
            action,
            drawn: 0,
            waited_secs: 0.0,
        }
    }
}

pub fn run_stroke_script(
    time: Res<Time>,
    status: Res<discovery::Status>,
    canvas: Res<starboard::canvas::Canvas>,
    brush: Res<starboard::input::OwnBrush>,
    mut script: ResMut<StrokeScript>,
    mut draws: MessageWriter<starboard::input::PendingLocalDraw>,
    mut is_logged: Local<bool>,
) {
    let time = time.into_inner();
    let status = status.into_inner();
    let canvas = canvas.into_inner();
    let brush = brush.into_inner();
    if !starboard::room::is_in_room(status) {
        return;
    }
    script.waited_secs += time.delta_secs();
    if script.waited_secs < SCRIPT_START_SECONDS {
        return;
    }
    if matches!(script.action, ScriptArg::Erase(_)) && !canvas.tiles.iter().any(Option::is_some) {
        return;
    }
    let total = script.action.strokes();
    if script.drawn < total {
        let op = stroke_op(script.action, script.drawn, *brush);
        draws.write(starboard::input::PendingLocalDraw(op));
        script.drawn = script.drawn.saturating_add(1);
        return;
    }
    if !*is_logged {
        *is_logged = true;
        tracing::info!(
            "starboard script done {} strokes={total}",
            script.action.label()
        );
    }
}

// needed helper: one scripted stroke at a fixed cell, shared by the painter and the eraser
fn stroke_op(
    action: ScriptArg,
    index: u32,
    brush: starboard::input::OwnBrush,
) -> starboard::canvas::DrawOp {
    let step = u16::try_from(index).unwrap_or(0);
    let from = starboard::canvas::CellPos {
        x: 200_u16.saturating_add(step.saturating_mul(30)),
        y: 300_u16.saturating_add(step.saturating_mul(9)),
    };
    let to = starboard::canvas::CellPos {
        x: from.x.saturating_add(60),
        y: from.y,
    };
    let draw_action = match action {
        ScriptArg::Paint(_) => starboard::canvas::DrawAction::Paint(brush.color),
        ScriptArg::Erase(_) => starboard::canvas::DrawAction::Erase,
    };
    starboard::input::draw_op(brush, draw_action, from, to)
}

#[cfg(test)]
mod tests {
    use super::stroke_op;
    use crate::args::ScriptArg;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let brush = starboard::input::OwnBrush::default();
        let paint = stroke_op(ScriptArg::Paint(2), 1, brush);
        let erase = stroke_op(ScriptArg::Erase(2), 1, brush);

        assert_eq!(paint.from, erase.from);
        assert_eq!(paint.to, erase.to);
        assert!(matches!(
            paint.action,
            starboard::canvas::DrawAction::Paint(_)
        ));
        assert_eq!(erase.action, starboard::canvas::DrawAction::Erase);
        assert_ne!(
            stroke_op(ScriptArg::Paint(2), 0, brush).from,
            stroke_op(ScriptArg::Paint(2), 1, brush).from
        );
    }
}
