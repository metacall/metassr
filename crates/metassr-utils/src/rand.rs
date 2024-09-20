use std::{
    collections::hash_map::RandomState,
    fmt::Display,
    hash::{BuildHasher, Hasher},
};

/// `Rand` is a simple structure that generates a random 64-bit integer value.
/// It uses the `RandomState` from the standard library's hash map as a source of randomness.
///
/// This is useful for situations where you need a random integer in your application.
/// The random value is generated when the `Rand` struct is instantiated.
///
/// **Example**
///
/// ```rust
/// use metassr_utils::rand::Rand;
/// let rand = Rand::new();
/// println!("Generated random value: {}", rand.val());
/// ```
#[derive(Debug)]
pub struct Rand(i64);

impl Rand {
    /// Creates a new `Rand` instance, generating a random i64 value using `RandomState`.
    ///
    /// The generated value is always non-negative.
    ///
    /// **Example**
    ///
    /// ```rust
    /// use metassr_utils::rand::Rand;
    ///
    /// let random = Rand::new();
    /// assert!(random.val() >= 0, "Random value should be non-negative");
    /// ```
    pub fn new() -> Self {
        let val = RandomState::new().build_hasher().finish() as i64; // Generate a random value
        Self(val.abs()) // Ensure the value is non-negative
    }

    /// Returns the generated random value.
    ///
    /// **Example**
    ///
    /// ```rust
    /// use metassr_utils::rand::Rand;
    ///
    /// let random = Rand::new();
    /// println!("Generated value: {}", random.val());
    /// ```
    pub fn val(&self) -> i64 {
        self.0 // Return the stored random value
    }
}

impl Display for Rand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl PartialEq for Rand {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}

impl PartialOrd for Rand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
    fn ge(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
    fn gt(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0
    }
    fn lt(&self, other: &Self) -> bool {
        self.0 < other.0
    }
}

impl PartialEq<i64> for Rand {
    fn eq(&self, other: &i64) -> bool {
        &self.0 == other
    }
    fn ne(&self, other: &i64) -> bool {
        &self.0 != other
    }
}

impl PartialOrd<i64> for Rand {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
    fn ge(&self, other: &i64) -> bool {
        &self.0 >= other
    }
    fn gt(&self, other: &i64) -> bool {
        &self.0 > other
    }

    fn le(&self, other: &i64) -> bool {
        &self.0 <= other
    }
    fn lt(&self, other: &i64) -> bool {
        &self.0 < other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test case to verify that `Rand::new()` generates a non-negative value.
    #[test]
    fn test_random_value_is_non_negative() {
        let random = Rand::new();
        assert!(
            random >= 0,
            "Expected non-negative random value, got {}",
            random
        );
    }

    /// Test case to ensure random values are distinct in multiple instantiations.
    #[test]
    fn test_random_values_are_distinct() {
        let mut values: Vec<Rand> = vec![];

        // Generate 10 random values and check for uniqueness
        for _ in 0..10 {
            let val = Rand::new();
            assert!(!values.contains(&val), "Random value repeated: {}", val);
            values.push(val);
        }

        println!("Generated random values: {:?}", values);
    }

    /// Test case to verify multiple random values over several iterations
    #[test]
    fn test_random_value_repetition_over_iterations() {
        let iterations = 100;
        let mut values: Vec<Rand> = vec![];

        // Generate a large number of random values to verify low repetition rate
        for _ in 0..iterations {
            let val = Rand::new();
            assert!(!values.contains(&val), "Repeated random value: {}", val);
            values.push(val);
        }

        println!("Generated {} random values: {:?}", iterations, values);
    }
}
