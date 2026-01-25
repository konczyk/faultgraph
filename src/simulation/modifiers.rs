#[derive(Clone)]
pub struct Throttle {
    factor: f64,
    default: f64,
    active: bool,
    just_applied: bool,
}

impl Throttle {
    pub fn new() -> Self {
        Self {
            factor: 1.0,
            default: 1.0,
            active: false,
            just_applied: false,
        }
    }

    pub fn factor(&self) -> f64 {
        if self.active {
            self.factor
        } else {
            self.default
        }
    }

    pub fn apply(&mut self, factor: f64) {
        if self.factor != factor {
            self.factor = factor;
            self.active = true;
            self.just_applied = true;
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn deactivate(&mut self) {
        self.factor = self.default;
        self.active = false;
        self.just_applied = false;
    }

    pub fn is_just_applied(&self) -> bool {
        self.just_applied
    }

    pub fn reset_just_applied(&mut self) {
        self.just_applied = false;
    }
}
