//! Helper logic for the [`Suffix`] type

use clap::{Arg, ArgGroup, ArgMatches, Command};
use indexer_rabbitmq::suffix::Suffix;

/// Wrapper around a [`Suffix`] providing `clap` support
#[derive(Debug)]
#[repr(transparent)]
pub struct QueueSuffix(Suffix);

impl From<Suffix> for QueueSuffix {
    #[inline]
    fn from(val: Suffix) -> Self {
        Self(val)
    }
}

impl From<QueueSuffix> for Suffix {
    #[inline]
    fn from(val: QueueSuffix) -> Self {
        val.0
    }
}

impl clap::Args for QueueSuffix {
    fn augment_args(cmd: Command) -> Command {
        cmd.arg(
            Arg::new("staging")
                .num_args(0)
                .value_parser(clap::builder::BoolishValueParser::new())
                .default_missing_value("true")
                .long("staging")
                .env("STAGING")
                .help("Use a staging queue suffix rather than a debug or production one"),
        )
        .arg(
            Arg::new("suffix")
                .num_args(1)
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .required(false)
                .help("An optional debug queue suffix")
                .conflicts_with("staging"),
        )
        .group(ArgGroup::new("Suffix").args(["staging", "suffix"]))
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
}

impl clap::FromArgMatches for QueueSuffix {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        Ok(if matches.get_one("staging").copied().unwrap_or_default() {
            Suffix::Staging
        } else if let Some(suffix) = matches.get_one("suffix") {
            Suffix::Debug(String::clone(suffix))
        } else {
            Suffix::Production
        }
        .into())
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::{Args, FromArgMatches};

    use super::{QueueSuffix, Suffix};

    fn parse<I: IntoIterator>(it: I) -> Result<Suffix, clap::Error>
    where
        I::Item: Into<std::ffi::OsString> + Clone,
    {
        let cmd = clap::Command::new("rmq-test");
        let cmd = QueueSuffix::augment_args(cmd);
        let matches = cmd.try_get_matches_from(it)?;

        QueueSuffix::from_arg_matches(&matches).map(Into::into)
    }

    #[test]
    fn test_suffix() {
        assert!(matches!(parse(["test", "--staging"]), Ok(Suffix::Staging)));

        assert!(matches!(parse(["test", "test"]), Ok(Suffix::Debug(_))));
        if let Ok(Suffix::Debug(d)) = parse(["test"]) {
            assert_eq!(d, "test");
        }

        assert!(matches!(parse(["test"]), Ok(Suffix::Production)));
    }
}
