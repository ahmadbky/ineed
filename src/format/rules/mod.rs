//! Module exposing various types of rules, discriminated by prompt kinds.

mod selected;
mod then;
mod written;

pub use selected::*;
pub use then::*;
pub use written::*;
