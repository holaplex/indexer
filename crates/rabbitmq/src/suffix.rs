//! Support logic for configuring queues with suffixed names

use std::fmt::Write;

use clap::{Arg, ArgMatches, Command};

use crate::{Error, Result};

/// A suffix for an AMQP object, to avoid name collisions with staging or debug
/// builds
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suffix {
    /// This is a production name
    Production,
    /// This is a staging name, to be treated similarly to production
    Staging,
    /// This is a debug name, identified further with a unique name
    Debug(String),
}

impl clap::Args for Suffix {
    fn augment_args(cmd: Command) -> Command {
        cmd.arg(
            Arg::new("STAGING")
                .long("staging")
                .env("STAGING")
                .takes_value(false)
                .help("Use a staging queue suffix rather than a debug or production one"),
        )
        .arg(
            Arg::new("SUFFIX")
                .takes_value(true)
                .allow_invalid_utf8(true)
                .required(false)
                .help("An optional debug queue suffix")
                .conflicts_with("STAGING"),
        )
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
}

impl clap::FromArgMatches for Suffix {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        Ok(if matches.is_present("STAGING") {
            Self::Staging
        } else if let Some(suffix) = matches.value_of_lossy("SUFFIX") {
            Self::Debug(suffix.into_owned())
        } else {
            Self::Production
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

impl Suffix {
    #[inline]
    pub(crate) fn is_debug(&self) -> bool {
        matches!(self, Self::Debug(_))
    }

    pub(crate) fn format(&self, mut prefix: String) -> Result<String> {
        if cfg!(debug_assertions) && !self.is_debug() {
            return Err(Error::InvalidQueueType(
                "Debug builds must specify a unique debug suffix for all AMQP names",
            ));
        }

        match self {
            Self::Production => (),
            Self::Staging => write!(prefix, ".staging").unwrap_or_else(|_| unreachable!()),
            Self::Debug(s) => write!(prefix, ".debug.{}", s).unwrap_or_else(|_| unreachable!()),
        }

        Ok(prefix)
    }
}
