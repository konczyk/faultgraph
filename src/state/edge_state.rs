#[derive(Clone, Copy)]
pub struct EdgeState {
    enabled: bool
}

impl EdgeState {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}