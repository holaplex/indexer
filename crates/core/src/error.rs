//! Utility types for propagating errors.

/// Alias for the `anyhow` error type
pub type Error = anyhow::Error;
/// Result alias that defaults to using the provided `Error` alias
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) mod prelude {
    pub use anyhow::{anyhow, bail, ensure, Context};

    pub use super::{Error, Result};
}
