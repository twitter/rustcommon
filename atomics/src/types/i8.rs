// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

native!(
    /// A signed 8 bit integer which can be shared between threads
    pub struct AtomicI8: i8 = core::sync::atomic::AtomicI8;
);

// additional traits
arithmetic!(AtomicI8, i8);
bitwise!(AtomicI8, i8);
fetch_compare_store!(AtomicI8, i8);
saturating_arithmetic!(AtomicI8, i8);

impl Signed for AtomicI8 {}

#[cfg(feature = "serde")]
struct AtomicI8Visitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for AtomicI8Visitor {
    type Value = AtomicI8;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a signed 8bit integer")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::new(value))
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::convert::TryFrom;
        if let Ok(value) = i8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("i8 is out of range: {}", value)))
        }
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::convert::TryFrom;
        if let Ok(value) = i8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("i8 is out of range: {}", value)))
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::convert::TryFrom;
        if let Ok(value) = i8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("i8 is out of range: {}", value)))
        }
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::convert::TryFrom;
        if let Ok(value) = i8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("i8 is out of range: {}", value)))
        }
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::convert::TryFrom;
        if let Ok(value) = i8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("i8 is out of range: {}", value)))
        }
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::convert::TryFrom;
        if let Ok(value) = i8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("i8 is out of range: {}", value)))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::convert::TryFrom;
        if let Ok(value) = i8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("i8 is out of range: {}", value)))
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AtomicI8 {
    fn deserialize<D>(deserializer: D) -> Result<AtomicI8, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i8(AtomicI8Visitor)
    }
}

#[cfg(feature = "serde")]
impl Serialize for AtomicI8 {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_some(&self.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load() {
        let atomic = AtomicI8::new(0);
        assert_eq!(atomic.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn store() {
        let atomic = AtomicI8::new(0);
        atomic.store(1, Ordering::SeqCst);
        assert_eq!(atomic.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn swap() {
        let atomic = AtomicI8::new(0);
        assert_eq!(atomic.swap(1, Ordering::SeqCst), 0);
    }

    #[test]
    fn compare_exchange() {
        let atomic = AtomicI8::new(0);
        assert_eq!(
            atomic.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst),
            Ok(0)
        );
        assert_eq!(
            atomic.compare_exchange(0, 2, Ordering::SeqCst, Ordering::SeqCst),
            Err(1)
        );
    }

    #[test]
    fn compare_exchange_weak() {
        let atomic = AtomicI8::new(0);
        loop {
            if atomic
                .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break;
            }
        }
        assert_eq!(atomic.load(Ordering::SeqCst), 1);
    }
}
