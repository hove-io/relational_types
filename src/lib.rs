//! Modeling the relations between objects.
//!
//! By default, feature `relations_procmacro` is enabled, exposing macros to
//! help build relations. See documentation of the crate `relations_procmacro`
//! for more information.

mod error;
mod relations;

pub use crate::error::*;
pub use crate::relations::*;
#[cfg(feature = "relations_procmacro")]
pub use relations_procmacro::*;
