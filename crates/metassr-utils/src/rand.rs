use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
};

/// A very simple random implementation for specific purposes (Just get a random value).

pub struct Rand(i64);

impl Rand {
    pub fn new() -> Self {
        let val = RandomState::new().build_hasher().finish() as i64;
        Self(val.abs())
    }
    pub fn val(&self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::rand::Rand;

    #[test]
    fn random_value() {
        let mut values: Vec<i64> = vec![];
        for _ in 1..10 {
            let val = Rand::new().val();
            if values.contains(&val) {
                panic!("a random value repeated");
            }
            values.push(val);
        }
        println!("{values:?}")
    }
}
