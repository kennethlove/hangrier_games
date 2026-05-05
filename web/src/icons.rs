//! Public icon API. The codegen module is private; consumers go through here.

#[path = "icons_generated.rs"]
mod generated;

pub use generated::*;
