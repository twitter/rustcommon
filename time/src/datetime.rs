// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

use time::OffsetDateTime;

pub enum SecondsFormat {
    Secs,
    Millis,
    Micros,
    Nanos,
}

#[derive(Copy, Clone)]
/// Represents a fixed moment in time in a format that has a human
/// representation.
///
/// It is important to note that the underlying clock is subject to phase and
/// frequency adjustments. This means that it is not guaranteed to be stable or
/// monotonically non-decreasing.
pub struct DateTime {
    pub(crate) inner: OffsetDateTime,
}

impl From<UnixInstant<Nanoseconds<u64>>> for DateTime {
    fn from(other: UnixInstant<Nanoseconds<u64>>) -> Self {
        let seconds = other.inner.inner / NANOS_PER_SEC;
        let nanoseconds = other.inner.inner % NANOS_PER_SEC;
        DateTime {
            inner: OffsetDateTime::from_unix_timestamp(seconds as i64).unwrap()
                + time::Duration::nanoseconds(nanoseconds as i64),
        }
    }
}

impl DateTime {
    pub fn now() -> Self {
        Self::from(UnixInstant::<Nanoseconds<u64>>::now())
    }

    pub fn recent() -> Self {
        Self::from(UnixInstant::<Nanoseconds<u64>>::recent())
    }

    pub fn to_rfc3339_opts(&self, seconds_format: SecondsFormat, use_z: bool) -> String {
        let date = self.inner.date();
        let time = self.inner.time();
        let tz = if use_z { "Z" } else { "+00:00" };
        let seconds = match seconds_format {
            SecondsFormat::Secs => {
                format!("{:02}", time.second())
            }
            SecondsFormat::Millis => {
                format!("{:02}.{:03}", time.second(), time.millisecond())
            }
            SecondsFormat::Micros => {
                format!("{:02}.{:06}", time.second(), time.microsecond())
            }
            SecondsFormat::Nanos => {
                format!("{:02}.{:09}", time.second(), time.nanosecond())
            }
        };
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{}{}",
            date.year(),
            date.month() as u8,
            date.day(),
            time.hour(),
            time.minute(),
            seconds,
            tz
        )
    }
}

impl core::ops::Add<core::time::Duration> for DateTime {
    type Output = DateTime;
    fn add(
        self,
        rhs: core::time::Duration,
    ) -> <Self as std::ops::Add<core::time::Duration>>::Output {
        DateTime {
            inner: self.inner + rhs,
        }
    }
}

impl core::ops::Sub<core::time::Duration> for DateTime {
    type Output = DateTime;
    fn sub(
        self,
        rhs: core::time::Duration,
    ) -> <Self as std::ops::Sub<core::time::Duration>>::Output {
        DateTime {
            inner: self.inner - rhs,
        }
    }
}

impl core::fmt::Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let date = self.inner.date();
        let time = self.inner.time();

        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            date.year(),
            date.month() as u8,
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
            time.millisecond()
        )
    }
}
