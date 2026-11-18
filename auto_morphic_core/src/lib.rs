//! Core traits and types for **AutoMorphic** — zero-cost struct-to-struct mapping.
//!
//! This crate provides the foundation traits used by the `auto_morphic` derive macro:
//!
//! - [`Morph<T>`] — bidirectional infallible conversion between two types.
//! - [`MorphResult<T>`] — bidirectional fallible conversion (returns `Result`).
//! - [`MorphError`] — error type with per-field messages.
//!
//! Most users should depend on the [`auto_morphic`](https://crates.io/crates/auto_morphic) crate,
//! which re-exports everything here and provides `#[derive(AutoMorph)]`.
//!
//! # Example
//!
//! ```
//! use auto_morphic_core::{Morph, MorphResult, MorphError};
//!
//! // The Morph trait defines two methods:
//! //   fn morph_to(&self) -> T;
//! //   fn morph_from(t: T) -> Self;
//! ```

pub mod morph;

// Re-export main types for convenience
pub use morph::{Morph, MorphResult, MorphError};
