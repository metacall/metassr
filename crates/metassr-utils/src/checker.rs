/// A simple checker state
#[derive(Debug)]
pub struct CheckerState(bool);
impl CheckerState {
    pub fn new() -> Self {
        Self(false)
    }

    pub fn with(state: bool) -> Self {
        Self(state)
    }

    pub fn make_true(&mut self) {
        self.0 = true
    }

    pub fn make_false(&mut self) {
        self.0 = false
    }

    pub fn is_true(&self) -> bool {
        self.0
    }
}

impl Default for CheckerState {
    fn default() -> Self {
        Self::new()
    }
}
