use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
};

/// `Rand` is a simple structure that generates a random 64-bit integer value.
/// It uses the `RandomState` from the standard library's hash map as a source of randomness.
/// 
/// This is useful for situations where you need a random integer in your application.
/// The random value is generated when the `Rand` struct is instantiated.
///
/// # Example
/// 
/// ```rust
/// use metassr_utils::rand::Rand;
/// let rand = Rand::new();
/// println!("Generated random value: {}", rand.val());
/// ```
pub struct Rand(i64);

impl Rand {
    /// Creates a new `Rand` instance, generating a random i64 value using `RandomState`.
    /// 
    /// The generated value is always non-negative.
    ///
    /// # Example
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
    /// # Example
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test case to verify that `Rand::new()` generates a non-negative value.
    #[test]
    fn test_random_value_is_non_negative() {
        let random = Rand::new();
        assert!(random.val() >= 0, "Expected non-negative random value, got {}", random.val());
    }

    /// Test case to ensure random values are distinct in multiple instantiations.
    #[test]
    fn test_random_values_are_distinct() {
        let mut values: Vec<i64> = vec![];
        
        // Generate 10 random values and check for uniqueness
        for _ in 0..10 {
            let val = Rand::new().val();
            assert!(!values.contains(&val), "Random value repeated: {}", val);
            values.push(val);
        }
        
        println!("Generated random values: {:?}", values);
    }

    /// Test case to verify multiple random values over several iterations
    #[test]
    fn test_random_value_repetition_over_iterations() {
        let iterations = 100;
        let mut values: Vec<i64> = vec![];

        // Generate a large number of random values to verify low repetition rate
        for _ in 0..iterations {
            let val = Rand::new().val();
            assert!(!values.contains(&val), "Repeated random value: {}", val);
            values.push(val);
        }

        println!("Generated {} random values: {:?}", iterations, values);
    }
}
