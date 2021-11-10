pub type Error = anyhow::Error;

pub(crate) mod prelude {
    pub use anyhow::Context;

    pub use super::Error;
}
