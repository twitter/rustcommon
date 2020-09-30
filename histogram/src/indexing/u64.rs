// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

impl crate::Indexing for u64 {
    fn constrain_precision(precision: u8) -> u8 {
        if precision == 0 {
            1
        } else if precision > 20 {
            20
        } else {
            precision
        }
    }

    fn constrain_exact(max: Self, precision: u8) -> Self {
        if precision == 20 {
            max
        } else {
            10_u64.pow(precision.into())
        }
    }

    fn get_index(value: Self, max: Self, exact: Self, precision: u8) -> Result<usize, ()> {
        if value > max {
            Err(())
        } else if value <= exact {
            Ok(value as usize)
        } else {
            let power = if value < 100 {
                1
            } else if value < 1_000 {
                2
            } else if value < 10_000 {
                3
            } else if value < 100_000 {
                4
            } else if value < 1_000_000 {
                5
            } else if value < 10_000_000 {
                6
            } else if value < 100_000_000 {
                7
            } else if value < 1_000_000_000 {
                8
            } else if value < 10_000_000_000 {
                9
            } else if value < 100_000_000_000 {
                10
            } else if value < 1_000_000_000_000 {
                11
            } else if value < 10_000_000_000_000 {
                12
            } else if value < 100_000_000_000_000 {
                13
            } else if value < 1_000_000_000_000_000 {
                14
            } else if value < 10_000_000_000_000_000 {
                15
            } else if value < 100_000_000_000_000_000 {
                16
            } else if value < 1_000_000_000_000_000_000 {
                17
            } else if value < 10_000_000_000_000_000_000 {
                18
            } else {
                19
            };
            let denominator = 10_usize.pow((power - precision as u32 + 1).into());
            let power_offset = 9 * exact as usize * (power as usize - precision as usize) / 10;
            let remainder: usize = value as usize / denominator;
            let shift = exact as usize / 10;
            let index = exact as usize + power_offset + remainder - shift;
            Ok(index)
        }
    }

    // Internal function to get the minimum value for a given bucket index
    fn get_min_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()> {
        if index >= buckets {
            Err(())
        } else if (index as u64) <= exact {
            Ok(index as u64)
        } else if index == buckets - 1 {
            Ok(max)
        } else {
            let base_offset = 10_usize.pow(precision.into());
            let shift = base_offset / 10;
            let power = precision as usize + (index - base_offset) / (9 * shift);
            let power_offset = 9 * (shift * (power - precision as usize));
            let value = (index + shift - base_offset - power_offset) as u64
                * 10_u64.pow((power - precision as usize + 1) as u32);
            Ok(value)
        }
    }

    // Internal function to get the max value stored in a given bucket
    fn get_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()> {
        if index == buckets - 1 {
            Ok(max)
        } else {
            Ok(Self::get_min_value(index + 1, buckets, max, exact, precision)? - 1)
        }
    }

    fn get_max_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()> {
        Self::get_value(index, buckets, max, exact, precision).map(|v| v + 1)
    }
}
