use std::time::{SystemTime, UNIX_EPOCH};

pub struct Tick {
    value: u32,
}

impl Tick {
    pub fn now() -> Self {
        let tick: u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let tick = (tick / 10) as u32;

        Self {
            value: tick & 0x7FFFFFFF,
        }
    }

    pub fn new(value: u32) -> Self {
        Self {
            value: value & 0x7FFFFFFF,
        }
    }

    pub fn diff(&self, other: &Tick) -> i32 {
        let first: i32 = (self.value << 1) as i32;
        let second: i32 = (other.value << 1) as i32;

        (first - second) >> 1
    }

    pub fn gt(&self, other: &Tick) -> bool {
        self.diff(other) > 0
    }

    pub fn gte(&self, other: &Tick) -> bool {
        self.diff(other) >= 0
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}
