#[derive(Clone)]
pub struct CapacityModifier {
    factor: f64,
    active: bool,
    just_applied: bool,
    turns: u8,
    remaining: u8,
}

impl CapacityModifier {
    const BASELINE_FACTOR: f64 = 1.0;

    pub fn new() -> Self {
        Self {
            factor: 1.0,
            active: false,
            just_applied: false,
            turns: 3,
            remaining: 0,
        }
    }

    pub fn factor(&self) -> f64 {
        if self.active {
            self.factor
        } else {
            Self::BASELINE_FACTOR
        }
    }

    pub fn apply(&mut self, factor: f64) -> bool {
        if self.is_active() {
            return false;
        }
        self.factor = factor;
        self.active = true;
        self.just_applied = true;
        self.remaining = self.turns;
        true
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn deactivate(&mut self) {
        self.factor = Self::BASELINE_FACTOR;
        self.active = false;
        self.just_applied = false;
        self.remaining = 0;
    }

    pub fn is_just_applied(&self) -> bool {
        self.just_applied
    }

    pub fn tick(&mut self) {
        if !self.is_active() {
            return;
        }

        if self.just_applied {
            self.just_applied = false;
            return;
        }

        if self.remaining > 0 {
            self.remaining -= 1;
        }
        if self.remaining == 0 {
            self.deactivate();
        }
    }

    pub fn remaining_turns(&self) -> u8 {
        self.remaining
    }
}
