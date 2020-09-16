// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod u16;
mod u32;
mod u64;
mod u8;

/// Used to restrict what types may be used as values for histograms. Also used
/// to provide a unified interface for performing type-specific operations such
/// as indexing into the internal storage, converting an index back to bucket
/// values, and calculating configuration parameters for the histogram.
pub trait Indexing
where
    Self: Sized + Copy,
{
    /// Calculate a bucket index for a given value and configuration.
    fn get_index(value: Self, max: Self, exact: Self, precision: u8) -> Result<usize, ()>;

    /// Calculate the minimum stored value for a given bucket index and
    /// configuration.
    fn get_min_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()>;

    /// Calculate the nominal value for a given bucket index and configuration.
    fn get_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()>;

    /// Calculate the exclusive upper bound for a given bucket index and
    /// configuration.
    fn get_max_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()>;

    /// Used to reduce the configured precision based on the type.
    fn constrain_precision(precision: u8) -> u8;

    /// Used to calculate the highest value which is stored exactly for a given
    /// type and configuration.
    fn constrain_exact(max: Self, precision: u8) -> Self;
}
