use libp2p::gossipsub;

#[must_use]
pub fn gossipsub_message_event(
    propagation_source: libp2p::PeerId,
    source: Option<libp2p::PeerId>,
    data: Vec<u8>,
    topic: gossipsub::TopicHash,
) -> gossipsub::Event {
    gossipsub::Event::Message {
        propagation_source,
        message_id: gossipsub::MessageId::new(b"m1"),
        message: gossipsub::Message {
            source,
            data,
            sequence_number: None,
            topic,
        },
    }
}

#[cfg(test)]
mod tests {
    use libp2p::gossipsub;

    use super::gossipsub_message_event;

    #[test]
    fn test_usage() {
        let source = libp2p::PeerId::random();
        let topic = gossipsub::TopicHash::from_raw("global/topic");
        let event = gossipsub_message_event(
            libp2p::PeerId::random(),
            Some(source),
            vec![7],
            topic.clone(),
        );
        let gossipsub::Event::Message { message, .. } = event else {
            panic!("expected a message event");
        };
        assert_eq!(message.source, Some(source));
        assert_eq!((message.data, message.topic), (vec![7], topic));
    }
}
