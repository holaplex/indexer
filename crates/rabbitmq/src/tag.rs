use std::{fmt, fmt::Display};

use rand::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Tag<T>(T, u16);

impl<T> Tag<T> {
    #[inline]
    pub fn new(val: T) -> Self {
        Self(val, rand::thread_rng().gen())
    }
}

impl<T> From<T> for Tag<T> {
    #[inline]
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<T: Display> Display for Tag<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{}-{:04x}", self.0, self.1))
    }
}

#[inline]
pub fn tag(val: impl Display) -> String {
    Tag::new(val).to_string()
}
