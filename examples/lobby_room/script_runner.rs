use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::ping;
use crate::shared::script::poll_shared_script_step::{SharedStepView, poll_shared_script_step};
use crate::shared::script::script_runner::{ScriptRunner, app_exit_for_verdict};
use crate::shared::script::shared_script_step::SharedScriptStep;
use crate::shared::script::shared_step_effect_sink::SharedStepEffectSink;

pub type LobbyScriptRunner = ScriptRunner<SharedScriptStep>;

pub fn run_script(
    time: Res<Time>,
    status: Res<discovery::Status>,
    join_error: Option<Res<discovery::ui::JoinError>>,
    senders: Res<ping::RoomMessageSenders>,
    mut runner: ResMut<LobbyScriptRunner>,
    mut effects: SharedStepEffectSink,
    mut exits: MessageWriter<AppExit>,
) {
    let view = SharedStepView {
        status: status.into_inner(),
        join_error: join_error
            .map(Res::into_inner)
            .and_then(|error| error.as_deref()),
        room_message_senders: senders.into_inner().peers.len(),
    };
    let verdict = runner.advance(time.into_inner().delta(), |step, clock| {
        let poll = poll_shared_script_step(step, clock, &view);
        if let Some(effect) = poll.effect {
            effects.apply(effect);
        }
        poll.outcome
    });
    if let Some(exit) = app_exit_for_verdict(verdict) {
        exits.write(exit);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::discovery;

    use super::{LobbyScriptRunner, run_script};
    use crate::ping::RoomMessageSenders;
    use crate::shared::intent::PendingAction;
    use crate::shared::script::shared_script_step::SharedScriptStep;
    use crate::shared::script::timed_script_step::parse_timed_script_step;

    #[test]
    fn test_usage() {
        let steps = ["create:room-1", "stay"]
            .iter()
            .map(|raw| parse_timed_script_step::<SharedScriptStep>(raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.init_resource::<RoomMessageSenders>();
        app.insert_resource(PendingAction::default());
        app.add_message::<discovery::Command>();
        app.insert_resource(LobbyScriptRunner::new(steps));
        app.add_systems(Update, run_script);
        app.update();
        assert_eq!(app.world().resource::<LobbyScriptRunner>().index, 1);
        assert!(matches!(
            app.world().resource::<PendingAction>().0,
            Some(crate::shared::action::Action::Create(_))
        ));
    }
}
