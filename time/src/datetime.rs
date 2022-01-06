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
/// representation. This time is based off of an estimation of Unix Time taken
/// from the `UnixInstant` type. This means that `DateTime`s will have a strict
/// ordering that matches real time ordering, but the accuracy of the real value
/// will depend on the phase and frequency accuracy of the system clock.
pub struct DateTime {
    pub(crate) inner: OffsetDateTime,
}

impl DateTime {
    pub fn now() -> Self {
        let now = UnixInstant::<Nanoseconds<u64>>::now();
        let seconds = now.inner.inner / NANOS_PER_SEC;
        let nanoseconds = now.inner.inner % NANOS_PER_SEC;
        DateTime {
            inner: OffsetDateTime::from_unix_timestamp(seconds as i64).unwrap()
                + time::Duration::nanoseconds(nanoseconds as i64),
        }
    }

    pub fn recent() -> Self {
        let recent = UnixInstant::<Nanoseconds<u64>>::recent();
        let seconds = recent.inner.inner / NANOS_PER_SEC;
        let nanoseconds = recent.inner.inner % NANOS_PER_SEC;
        DateTime {
            inner: OffsetDateTime::from_unix_timestamp(seconds as i64).unwrap()
                + time::Duration::nanoseconds(nanoseconds as i64),
        }
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
