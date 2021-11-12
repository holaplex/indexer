pub type Error = anyhow::Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) mod prelude {
    pub use anyhow::{anyhow, bail, ensure, Context};

    pub use super::{Error, Result};
}
