//! Types related to dimensions/chunks.

pub mod generators;

pub mod dimension;
pub mod block;

pub use self::block::{BlockID, BlockName};
pub use self::dimension::Dimension;
