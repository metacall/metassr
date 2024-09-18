#![doc = include_str!("../README.md")]

/// This module provides directory analyzers for extracting useful files from directories like `src/` and `dist/`.
pub mod analyzer;

/// This module provides utilities for managing cache directories, creating files, and handling their contents.
/// 
/// This is useful for caching purposes where files are written and
/// retrieved based on a pathname, making it easier to manage multiple
/// cached files in a structured directory.
///
/// # Example
///
/// ```no_run
/// use metassr_utils::cache_dir::CacheDir;
///
/// let mut cache = CacheDir::new(".cache").unwrap();
/// cache.insert("example.txt", "Cache data".as_bytes()).unwrap();
/// println!("{:?}", cache.entries_in_scope());
/// ```
pub mod cache_dir;

/// This module contains a simple utility for managing a boolean state that can be toggled on or off.
/// It allows for easy manipulation of the state with utility methods to set it to `true` or `false`.
///
/// This can be useful in situations where you need a simple flag to represent a state in an application.
///
/// # Example
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
pub mod checker;

/// This module offers a utility to generate random numbers based on hash values for purposes such as creating random directory names.
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
pub mod rand;
