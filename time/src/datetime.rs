// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Duration;
use core::fmt::Display;
use core::fmt::Formatter;
use core::ops::Add;
use core::ops::Sub;
use time::OffsetDateTime;

pub enum SecondsFormat {
    Secs,
    Millis,
    Micros,
    Nanos,
}

#[derive(Copy, Clone)]
pub struct DateTime {
    pub(crate) inner: OffsetDateTime,
}

impl DateTime {
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

impl Add<Duration> for DateTime {
    type Output = DateTime;
    fn add(self, rhs: Duration) -> <Self as std::ops::Add<Duration>>::Output {
        DateTime {
            inner: self.inner + core::time::Duration::from_nanos(rhs.as_nanos() as u64),
        }
    }
}

impl Sub<Duration> for DateTime {
    type Output = DateTime;
    fn sub(self, rhs: Duration) -> <Self as std::ops::Sub<Duration>>::Output {
        DateTime {
            inner: self.inner - core::time::Duration::from_nanos(rhs.as_nanos() as u64),
        }
    }
}

impl Add<core::time::Duration> for DateTime {
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

impl Sub<core::time::Duration> for DateTime {
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

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
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
