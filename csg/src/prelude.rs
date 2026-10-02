//! `csg::prelude`
//! Crate prelude file.
//! Should be be included in most files
//! ```
//! use crate::prelude::*;
//! ```

pub use crate::{vfs::traits::CasinoFS, errors::{GError, GResult, GeneralizeError, GLFWError, WGPUError}};