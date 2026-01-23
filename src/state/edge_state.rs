#[derive(Clone, Copy)]
pub struct EdgeState {
    enabled: bool
}

impl EdgeState {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}