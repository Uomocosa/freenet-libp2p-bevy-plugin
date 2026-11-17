use crate::identity;
use crate::spectator;

pub fn release_topic_subscription<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    topic: &identity::Topic,
) -> bool {
    let Some(count) = machine.topic_subscriptions.get_mut(topic) else {
        return false;
    };
    *count = count.saturating_sub(1);
    if *count > 0 {
        return false;
    }
    machine.topic_subscriptions.remove(topic);
    true
}

#[cfg(test)]
mod tests {
    use super::release_topic_subscription;
    use crate::identity;
    use crate::spectator;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        let topic = identity::Topic::from("game/zone");
        assert!(!release_topic_subscription(&mut machine, &topic));
        assert!(machine.note_topic_subscription(&topic));
        assert!(release_topic_subscription(&mut machine, &topic));
        assert!(machine.topic_subscriptions.is_empty());
    }
}
