use atomic_delegate_macros::atomic_delegates;

use crate::testing;

pub struct KeysBuilder<'a> {
    pub(crate) client: &'a testing::harness::BrpClient,
    pub(crate) deadlines: testing::harness::Deadlines,
    pub(crate) steps: Vec<testing::KeyStep>,
}

impl<'a> KeysBuilder<'a> {
    #[must_use]
    pub const fn new(
        client: &'a testing::harness::BrpClient,
        deadlines: testing::harness::Deadlines,
    ) -> Self {
        Self {
            client,
            deadlines,
            steps: Vec::new(),
        }
    }
}

impl KeysBuilder<'_> {
    #[must_use]
    pub fn text(mut self, text: &str) -> Self {
        self.steps.push(testing::KeyStep::Text(text.to_owned()));
        self
    }

    #[must_use]
    pub fn tap(mut self, key: testing::Key) -> Self {
        self.steps.push(testing::KeyStep::Tap(key));
        self
    }

    #[must_use]
    pub fn chord(mut self, keys: impl IntoIterator<Item = testing::Key>) -> Self {
        self.steps
            .push(testing::KeyStep::Chord(keys.into_iter().collect()));
        self
    }

    #[must_use]
    pub fn hold_for_at_least(mut self, key: testing::Key, held_for: testing::Millis) -> Self {
        self.steps.extend([
            testing::KeyStep::Down(key),
            testing::KeyStep::Wait(held_for),
            testing::KeyStep::Up(key),
        ]);
        self
    }

    #[must_use]
    pub fn repeat(mut self, key: testing::Key, count: u32) -> Self {
        self.steps.push(testing::KeyStep::Repeat {
            key,
            count: testing::RepeatCount(count),
        });
        self
    }

    #[must_use]
    pub fn same_frame(mut self, script: impl FnOnce(Self) -> Self) -> Self {
        let inner = script(Self::new(self.client, self.deadlines));
        self.steps.push(testing::KeyStep::SameFrame(inner.steps));
        self
    }
}

#[atomic_delegates]
impl KeysBuilder<'_> {
    pub async fn send(self) -> Result<(), String> {}
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::KeysBuilder;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let client = testing::harness::BrpClient::new(1);
        let builder = KeysBuilder::new(&client, testing::harness::Deadlines(1.0))
            .text("hello")
            .tap(testing::Key::Enter)
            .chord([testing::Key::Ctrl, testing::Key::Char('z')])
            .hold_for_at_least(testing::Key::Char('w'), testing::Millis(500))
            .repeat(testing::Key::ArrowRight, 5)
            .same_frame(|keys| {
                keys.tap(testing::Key::Char(' '))
                    .tap(testing::Key::Char(' '))
            });
        assert_eq!(
            serde_json::to_value(&builder.steps).expect("steps serialize"),
            json!([
                {"Text": "hello"},
                {"Tap": "Enter"},
                {"Chord": ["Ctrl", {"Char": "z"}]},
                {"Down": {"Char": "w"}},
                {"Wait": 500},
                {"Up": {"Char": "w"}},
                {"Repeat": {"key": "ArrowRight", "count": 5}},
                {"SameFrame": [{"Tap": {"Char": " "}}, {"Tap": {"Char": " "}}]},
            ])
        );
        assert!(builder.send().await.is_err());
    }
}
