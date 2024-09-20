/// `CheckerState` is a simple structure that represents a boolean state.
/// It allows for easy manipulation of the state with utility methods to set it to `true` or `false`.
///
/// This can be useful in situations where you need a simple flag to represent a state in an application.
///
/// **Example**
///
/// ```rust
/// use metassr_utils::checker::CheckerState;
///
/// let mut state = CheckerState::default();
/// assert!(!state.is_true()); // Initially false by default
///
/// state.make_true();
/// assert!(state.is_true());  // Now true
///
/// state.make_false();
/// assert!(!state.is_true()); // Back to false
/// ```
#[derive(Debug)]
pub struct CheckerState(bool); // A tuple struct that holds a boolean value

impl CheckerState {
    /// Creates a new `CheckerState` with the specified boolean value.
    ///
    /// **Example**
    ///
    /// ```rust
    /// use metassr_utils::checker::CheckerState;
    ///
    /// let state = CheckerState::new(true);
    /// assert_eq!(state.is_true(), true);
    /// ```
    pub fn new(state: bool) -> Self {
        Self(state) // Allows for custom initialization with `true` or `false`
    }

    /// Sets the internal state to `true`.
    ///
    /// **Example**
    ///
    /// ```rust
    /// use metassr_utils::checker::CheckerState;
    ///
    /// let mut state = CheckerState::default();
    /// state.make_true();
    /// assert_eq!(state.is_true(), true);
    /// ```
    pub fn make_true(&mut self) {
        self.0 = true; // Mutate the internal state to true
    }

    /// Sets the internal state to `false`.
    ///
    /// **Example**
    ///
    /// ```rust
    /// use metassr_utils::checker::CheckerState;
    ///
    /// let mut state = CheckerState::new(true);
    /// state.make_false();
    /// assert_eq!(state.is_true(), false);
    /// ```
    pub fn make_false(&mut self) {
        self.0 = false; // Mutate the internal state to false
    }

    /// Returns `true` if the internal state is true, otherwise returns `false`.
    ///
    /// **Example**
    ///
    /// ```rust
    /// use metassr_utils::checker::CheckerState;
    ///
    /// let state = CheckerState::new(true);
    /// assert_eq!(state.is_true(), true);
    /// ```
    pub fn is_true(&self) -> bool {
        self.0 // Return the current state
    }
}

impl Default for CheckerState {
    /// The default implementation for `CheckerState`, which initializes the state to `false`.
    ///
    /// **Example**
    ///
    /// ```rust
    /// use metassr_utils::checker::CheckerState;
    ///
    /// let state = CheckerState::default();
    /// assert_eq!(state.is_true(), false);
    /// ```
    fn default() -> Self {
        Self::new(false) // By default, new CheckerState is false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test case for creating a new `CheckerState` using `new()`
    #[test]
    fn test_new_checker_state() {
        let state = CheckerState::default();
        assert!(!state.is_true(), "Expected state to be false");
    }

    /// Test case for creating a `CheckerState` with a specific boolean value
    #[test]
    fn test_with_checker_state() {
        let state_true = CheckerState::new(true);
        let state_false = CheckerState::new(false);

        assert!(state_true.is_true(), "Expected state to be true");
        assert!(!state_false.is_true(), "Expected state to be false");
    }

    /// Test case for the `make_true()` method
    #[test]
    fn test_make_true() {
        let mut state = CheckerState::new(false);
        state.make_true();
        assert!(
            state.is_true(),
            "Expected state to be true after calling make_true()"
        );
    }

    /// Test case for the `make_false()` method
    #[test]
    fn test_make_false() {
        let mut state = CheckerState::new(true);
        state.make_false();
        assert!(
            !state.is_true(),
            "Expected state to be false after calling make_false()"
        );
    }

    /// Test case for the default implementation
    #[test]
    fn test_default_checker_state() {
        let state: CheckerState = Default::default();
        assert!(!state.is_true(), "Expected default state to be false");
    }
}
