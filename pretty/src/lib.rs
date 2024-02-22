//! A pretty printing algorithm like Wadler's "A Prettier Printer",
//! but more expressive because it supports arbitrary choices.
//!
//! Nitty gritty details:
//!
//! - Supports fixed alignment but not dynamic alignment (i.e., you can't align
//!   something to match the position of what's above it, you can only indent it
//!   by fixed amounts). Just like Wadler's printer.
//! - Allows choices between _arbitrary_ alternatives, including alternatives
//!   that differ by more than just whitespace.
//! - Worst-case `O(nw)` (linear) running time, except in weird edge cases. Just
//!   like Wadler's printer.
//!
//! ## To test
//!
//! Install Rust if you haven't:
//!
//! ```
//! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
//! ```
//!
//! Run the Json printer:
//!
//! ```
//! cargo run --release --example json pokemon.json
//! ```

mod notation;
mod print;

pub use notation::{flat, indent, nl, txt, Notation};
pub use print::pretty_print;
