//! Ericsson Rust Library
//!
//! ## Features
//!
//! The library provides APIs for various interfaces, generic types and functionalities for easy
//! microservice development adhering to Ericsson requirements:
//! * Kafka
//! * _Others coming soon_
//!
//! ## Examples
//!
//! It is also a place to look for Rustacean ways to solve problems.
//!
//! [`types::SemVer`] is a place to look for an example for parsing things from strings with error
//! handling:
//! * Trait implementation for
//!   * [`std::str::FromStr`]
//!   * [`std::fmt::Display`]

pub mod kafka;

mod types;

pub use types::SemVer;
pub use types::Varint;
