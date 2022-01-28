// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

float!(
    /// A 32 bit floating point number which can be shared between threads
    pub struct AtomicF32: f32 = core::sync::atomic::AtomicU32;
);

// additional traits
float_arithmetic!(AtomicF32, f32);

#[cfg(feature = "serde")]
struct AtomicF32Visitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for AtomicF32Visitor {
    type Value = AtomicF32;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 32bit floating point number")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::new(f32::from(value)))
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::new(f32::from(value)))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = i16::try_from(value).map(f32::from) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("f32 is out of range: {}", value)))
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = i16::try_from(value).map(f32::from) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("f32 is out of range: {}", value)))
        }
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::new(f32::from(value)))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::new(f32::from(value)))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u16::try_from(value).map(f32::from) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("f32 is out of range: {}", value)))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = u16::try_from(value).map(f32::from) {
            Ok(Self::Value::new(value))
        } else {
            Err(E::custom(format!("f32 is out of range: {}", value)))
        }
    }

    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::new(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value >= f64::from(std::f32::MIN) && value <= f64::from(std::f32::MAX) {
            Ok(Self::Value::new(value as f32))
        } else {
            Err(E::custom(format!("f32 is out of range: {}", value)))
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AtomicF32 {
    fn deserialize<D>(deserializer: D) -> Result<AtomicF32, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AtomicF32Visitor)
    }
}

#[cfg(feature = "serde")]
impl Serialize for AtomicF32 {
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
        let atomic = AtomicF32::new(0.0);
        assert_eq!(atomic.load(Ordering::SeqCst), 0.0);
    }

    #[test]
    fn store() {
        let atomic = AtomicF32::new(0.0);
        atomic.store(std::f32::consts::PI, Ordering::SeqCst);
        assert_eq!(atomic.load(Ordering::SeqCst), std::f32::consts::PI);
    }

    #[test]
    fn swap() {
        let atomic = AtomicF32::new(0.0);
        assert_eq!(atomic.swap(std::f32::consts::PI, Ordering::SeqCst), 0.0);
    }

    #[test]
    fn compare_exchange() {
        let atomic = AtomicF32::new(0.0);
        assert_eq!(
            atomic.compare_exchange(0.0, std::f32::consts::PI, Ordering::SeqCst, Ordering::SeqCst),
            Ok(0.0)
        );
        assert_eq!(
            atomic.compare_exchange(0.0, 42.0, Ordering::SeqCst, Ordering::SeqCst),
            Err(std::f32::consts::PI)
        );
    }

    #[test]
    fn compare_exchange_weak() {
        let atomic = AtomicF32::new(0.0);
        loop {
            if atomic
                .compare_exchange(0.0, std::f32::consts::PI, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break;
            }
        }
        assert_eq!(atomic.load(Ordering::SeqCst), std::f32::consts::PI);
    }
}
