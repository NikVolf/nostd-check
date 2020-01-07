mod source;
mod meta;

#[cfg(test)]
mod tests;

pub use crate::source::{explore, Exploration};
pub use crate::meta::{find_all_root_libs, DependencySummary};