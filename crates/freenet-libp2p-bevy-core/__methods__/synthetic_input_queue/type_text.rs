use crate::testing;

pub fn type_text(queue: &mut testing::SyntheticInputQueue, text: &str) {
    queue
        .pending
        .extend(text.chars().map(testing::SyntheticInput::Key));
}

#[cfg(test)]
mod tests {
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::type_text(&mut queue, "ab");
        assert_eq!(queue.pending.len(), 2);
    }
}
