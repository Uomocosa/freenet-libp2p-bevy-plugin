#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Idle,
    Reveal {
        slot: usize,
        revealed_numbers: Vec<u32>,
        elapsed_secs: f32,
    },
    Roll {
        from: Vec<u32>,
        to: Vec<u32>,
        elapsed_secs: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Show { numbers: Vec<u32> },
    Tick { delta_secs: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    RevealStarted {
        slot: usize,
        value: u32,
    },
    RevealFrame {
        slot: usize,
        progress: f32,
    },
    RevealSettled {
        slot: usize,
        value: u32,
    },
    RollFrame {
        from: Vec<u32>,
        to: Vec<u32>,
        progress: f32,
    },
    RollSettled {
        numbers: Vec<u32>,
    },
}
