use std::{
    fmt::{self, Display},
    str::FromStr,
};

use anyhow::{anyhow, Context};
use target_lexicon::Triple;
use time::Date;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Toolchain {
    pub channel: Channel,
    pub date: Option<Date>,
    pub host: Option<Triple>,
}

impl Toolchain {
    #[inline]
    #[must_use]
    pub const fn new(channel: Channel, date: Option<Date>, host: Option<Triple>) -> Self {
        Self {
            channel,
            date,
            host,
        }
    }

    #[inline]
    #[must_use]
    pub const fn channel(channel: Channel) -> Self {
        Self {
            channel,
            date: None,
            host: None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_date(channel: Channel, date: Date) -> Self {
        Self {
            channel,
            date: Some(date),
            host: None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_host(channel: Channel, host: Triple) -> Self {
        Self {
            channel,
            date: None,
            host: Some(host),
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_date_and_host(channel: Channel, date: Date, host: Triple) -> Self {
        Self {
            channel,
            date: Some(date),
            host: Some(host),
        }
    }
}

impl FromStr for Toolchain {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((channel, s)) => {
                let (date, s) = if maybe_date(s) {
                    let (date, s) = s.split_at(10);
                    let (year, rest) = date.split_once('-').context("missing date year")?;
                    let (month, day) =
                        rest.split_once('-').context("missing date month and day")?;

                    let date = Date::from_calendar_date(
                        year.parse()?,
                        month.parse::<u8>()?.try_into()?,
                        day.parse()?,
                    )?;

                    (Some(date), s)
                } else {
                    (None, s)
                };

                let s = s.strip_prefix('-').unwrap_or(s);

                let host = (!s.is_empty())
                    .then(|| s.parse().map_err(|e| anyhow!("{}", e)))
                    .transpose()?;

                Ok(Self {
                    channel: channel.parse()?,
                    date,
                    host,
                })
            }
            None => Ok(Self {
                channel: s.parse()?,
                date: None,
                host: None,
            }),
        }
    }
}

impl Display for Toolchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.channel)?;

        if let Some(date) = &self.date {
            write!(f, "-{date}")?;
        }

        if let Some(host) = &self.host {
            write!(f, "-{host}")?;
        }

        Ok(())
    }
}

fn maybe_date(s: &str) -> bool {
    if s.len() < 10 {
        return false;
    }

    let sb = s.as_bytes();

    is_digit(&sb[0..=3])
        && sb[4] == b'-'
        && is_digit(&sb[5..=6])
        && sb[7] == b'-'
        && is_digit(&sb[8..=9])
}

fn is_digit(b: &[u8]) -> bool {
    b.iter().all(u8::is_ascii_digit)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
    Partial { major: u8, minor: u8 },
    Full { major: u8, minor: u8, patch: u8 },
}

impl FromStr for Channel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "stable" => Self::Stable,
            "beta" => Self::Beta,
            "nightly" => Self::Nightly,
            _ => {
                let mut parts = s.splitn(3, '.');
                let major = parts.next().context("missing major version")?.parse()?;
                let minor = parts.next().context("missing minor version")?.parse()?;

                if let Some(patch) = parts.next() {
                    Self::Full {
                        major,
                        minor,
                        patch: patch.parse()?,
                    }
                } else {
                    Self::Partial { major, minor }
                }
            }
        })
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable => f.write_str("stable"),
            Self::Beta => f.write_str("beta"),
            Self::Nightly => f.write_str("nightly"),
            Self::Partial { major, minor } => write!(f, "{major}.{minor}"),
            Self::Full {
                major,
                minor,
                patch,
            } => write!(f, "{major}.{minor}.{patch}"),
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::Stable
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use target_lexicon::{Architecture, BinaryFormat, Environment, OperatingSystem, Vendor};
    use time::macros::date;

    use super::*;

    #[test]
    fn test_toolchain() -> Result<()> {
        assert_eq!(Toolchain::channel(Channel::Stable), "stable".parse()?);
        assert_eq!(Toolchain::channel(Channel::Beta), "beta".parse()?);
        assert_eq!(Toolchain::channel(Channel::Nightly), "nightly".parse()?);
        assert_eq!(
            Toolchain::channel(Channel::Partial {
                major: 1,
                minor: 58
            }),
            "1.58".parse()?
        );
        assert_eq!(
            Toolchain::channel(Channel::Full {
                major: 1,
                minor: 58,
                patch: 1
            }),
            "1.58.1".parse()?
        );
        assert_eq!(
            Toolchain::with_date(Channel::Stable, date!(2022 - 01 - 01)),
            "stable-2022-01-01".parse()?
        );
        assert_eq!(
            Toolchain::with_host(
                Channel::Stable,
                Triple {
                    architecture: Architecture::X86_64,
                    vendor: Vendor::Unknown,
                    operating_system: OperatingSystem::Linux,
                    environment: Environment::Gnu,
                    binary_format: BinaryFormat::Elf
                }
            ),
            "stable-x86_64-unknown-linux-gnu".parse()?
        );
        assert_eq!(
            Toolchain::with_date_and_host(
                Channel::Stable,
                date!(2022 - 01 - 01),
                Triple {
                    architecture: Architecture::X86_64,
                    vendor: Vendor::Unknown,
                    operating_system: OperatingSystem::Linux,
                    environment: Environment::Gnu,
                    binary_format: BinaryFormat::Elf
                }
            ),
            "stable-2022-01-01-x86_64-unknown-linux-gnu".parse()?
        );
        assert_eq!(
            Toolchain::with_date_and_host(
                Channel::Full {
                    major: 1,
                    minor: 58,
                    patch: 1
                },
                date!(2022 - 01 - 01),
                Triple {
                    architecture: Architecture::X86_64,
                    vendor: Vendor::Unknown,
                    operating_system: OperatingSystem::Linux,
                    environment: Environment::Gnu,
                    binary_format: BinaryFormat::Elf
                }
            ),
            "1.58.1-2022-01-01-x86_64-unknown-linux-gnu".parse()?
        );

        Ok(())
    }
}
