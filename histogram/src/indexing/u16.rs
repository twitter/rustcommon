// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

impl crate::Indexing for u16 {
    fn constrain_precision(precision: u8) -> u8 {
        if precision == 0 {
            1
        } else if precision > 5 {
            5
        } else {
            precision
        }
    }

    fn constrain_exact(max: Self, precision: u8) -> Self {
        if precision == 5 {
            max
        } else {
            10_u16.pow(precision.into())
        }
    }

    fn get_index(value: Self, max: Self, exact: Self, precision: u8) -> Result<usize, ()> {
        if value > max {
            Err(())
        } else if value <= exact {
            Ok(value.into())
        } else {
            let power = if value < 10 {
                0
            } else if value < 100 {
                1
            } else if value < 1_000 {
                2
            } else if value < 10_000 {
                3
            } else {
                4
            };
            let denominator = 10_usize.pow((power - precision as u16 + 1).into());
            let power_offset =
                (0.9_f64 * f64::from(exact as u32 * (power as u32 - precision as u32))) as usize;
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
        } else if (index as u16) <= exact {
            Ok(index as u16)
        } else if index == buckets - 1 {
            Ok(max)
        } else {
            let shift = 10_usize.pow((precision - 1).into());
            let base_offset = 10_usize.pow(precision.into());
            let power = precision as usize + (index - base_offset) / (9 * shift);
            let power_offset = (0.9
                * (10_usize.pow(precision.into()) * (power - precision as usize)) as f64)
                as usize;
            let value = (index + shift - base_offset - power_offset) as u16
                * 10_u16.pow((power - precision as usize + 1) as u32);
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

#[cfg(test)]
mod tests {
    use crate::Indexing;

    #[test]
    fn get_index_p1() {
        let precision = 1;
        let max = u16::MAX;
        let exact = u16::constrain_exact(max, precision);
        for i in 0..(10_u16.pow(precision.into())) {
            assert_eq!(u16::get_index(i, max, exact, precision), Ok(i as usize));
        }
        for i in 1..10 {
            for j in 0..10 {
                let v = i * 10 + j;
                assert_eq!(u16::get_index(v, max, exact, precision), Ok(9 + i as usize));
            }
        }
        for i in 1..10 {
            for j in 0..100 {
                let v = i * 100 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(18 + i as usize)
                );
            }
        }
        for i in 1..10 {
            for j in 0..1000 {
                let v = i * 1000 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(27 + i as usize)
                );
            }
        }
        for i in 1..6 {
            for j in 0..10000 {
                let v = i * 10000 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(36 + i as usize)
                );
            }
        }
        for j in 0..5536 {
            let v = 60000 + j;
            assert_eq!(u16::get_index(v, max, exact, precision), Ok(42 as usize));
        }
    }

    #[test]
    fn get_index_p2() {
        let precision = 2;
        let max = u16::MAX;
        let exact = u16::constrain_exact(max, precision);
        for i in 0..(10_u16.pow(precision.into())) {
            assert_eq!(u16::get_index(i, max, exact, precision), Ok(i as usize));
        }
        for i in 10..100 {
            for j in 0..10 {
                let v = i * 10 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(90 + i as usize)
                );
            }
        }
        for i in 10..100 {
            for j in 0..100 {
                let v = i * 100 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(180 + i as usize)
                );
            }
        }
        for i in 10..65 {
            for j in 0..1000 {
                let v = i * 1000 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(270 + i as usize)
                );
            }
        }
        for j in 0..536 {
            let v = 65000 + j;
            assert_eq!(u16::get_index(v, max, exact, precision), Ok(335 as usize));
        }
    }

    #[test]
    fn get_index_p3() {
        let precision = 3;
        let max = u16::MAX;
        let exact = u16::constrain_exact(max, precision);
        for i in 0..(10_u16.pow(precision.into())) {
            assert_eq!(u16::get_index(i, max, exact, precision), Ok(i as usize));
        }
        for i in 100..1000 {
            for j in 0..10 {
                let v = i * 10 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(900 + i as usize)
                );
            }
        }
        for i in 100..655 {
            for j in 0..100 {
                let v = i * 100 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(1800 + i as usize)
                );
            }
        }
        for j in 0..36 {
            let v = 65500 + j;
            assert_eq!(u16::get_index(v, max, exact, precision), Ok(2455 as usize));
        }
    }

    #[test]
    fn get_index_p4() {
        let precision = 4;
        let max = u16::MAX;
        let exact = u16::constrain_exact(max, precision);
        for i in 0..(10_u16.pow(precision.into())) {
            assert_eq!(u16::get_index(i, max, exact, precision), Ok(i as usize));
        }
        for i in 1000..6553 {
            for j in 0..10 {
                let v = i * 10 + j;
                assert_eq!(
                    u16::get_index(v, max, exact, precision),
                    Ok(9000 + i as usize)
                );
            }
        }
        for j in 0..6 {
            let v = 65530 + j;
            assert_eq!(u16::get_index(v, max, exact, precision), Ok(15553 as usize));
        }
    }

    #[test]
    fn get_index_p5() {
        let precision = 5;
        let max = u16::MAX;
        let exact = u16::constrain_exact(max, precision);
        for v in 0..max {
            assert_eq!(u16::get_index(v, max, exact, precision), Ok(v as usize));
        }
    }
}
