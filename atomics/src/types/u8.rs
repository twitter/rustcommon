// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

native!(
    /// An unsigned 8 bit integer which can be shared between threads
    pub struct AtomicU8: u8 = core::sync::atomic::AtomicU8;
);

// additional traits
arithmetic!(AtomicU8, u8);
bitwise!(AtomicU8, u8);
fetch_compare_store!(AtomicU8, u8);
saturating_arithmetic!(AtomicU8, u8);

impl Unsigned for AtomicU8 {}

#[cfg(feature = "serde")]
struct AtomicU8Visitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for AtomicU8Visitor {
    type Value = AtomicU8;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an unsigned 8bit integer")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("u8 is out of range: {}", value)))
        }
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("u8 is out of range: {}", value)))
        }
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("u8 is out of range: {}", value)))
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("u8 is out of range: {}", value)))
        }
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::new(value))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("u8 is out of range: {}", value)))
        }
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("u8 is out of range: {}", value)))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u8::try_from(value) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("u8 is out of range: {}", value)))
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AtomicU8 {
    fn deserialize<D>(deserializer: D) -> Result<AtomicU8, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AtomicU8Visitor)
    }
}

#[cfg(feature = "serde")]
impl Serialize for AtomicU8 {
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
        let atomic = AtomicU8::new(0);
        assert_eq!(atomic.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn store() {
        let atomic = AtomicU8::new(0);
        atomic.store(1, Ordering::SeqCst);
        assert_eq!(atomic.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn swap() {
        let atomic = AtomicU8::new(0);
        assert_eq!(atomic.swap(1, Ordering::SeqCst), 0);
    }

    #[test]
    fn compare_exchange() {
        let atomic = AtomicU8::new(0);
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
        let atomic = AtomicU8::new(0);
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
