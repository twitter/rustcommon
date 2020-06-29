// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

native!(
    /// A boolean type which can be shared between threads
    pub struct AtomicBool: bool = core::sync::atomic::AtomicBool;
);

// additional traits
bitwise!(AtomicBool, bool);

#[cfg(feature = "serde")]
struct AtomicBoolVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for AtomicBoolVisitor {
    type Value = AtomicBool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a boolean value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(AtomicBool::new(value))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AtomicBool {
    fn deserialize<D>(deserializer: D) -> Result<AtomicBool, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bool(AtomicBoolVisitor)
    }
}

#[cfg(feature = "serde")]
impl Serialize for AtomicBool {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_some(&self.load(Ordering::SeqCst))
    }
}
