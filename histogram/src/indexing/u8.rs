impl crate::Indexing for u8 {
    fn constrain_precision(precision: u8) -> u8 {
        if precision == 0 {
            1
        } else if precision > 3 {
            3
        } else {
            precision
        }
    }

    fn constrain_exact(max: Self, precision: u8) -> Self {
        if precision == 3 {
            max
        } else {
            10_u8.pow(precision.into())
        }
    }

    fn get_index(value: Self, max: Self, exact: Self, precision: u8) -> Result<usize, ()> {
        if value > max {
            Err(())
        } else if value <= exact {
            Ok(value.into())
        } else {
            let power = (value as f64).log10().floor() as u8;
            let denominator = 10_usize.pow((power - precision + 1).into());
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
        } else if (index as u8) <= exact {
            Ok(index as u8)
        } else if index == buckets - 1 {
            Ok(max)
        } else {
            let shift = 10_usize.pow((precision - 1).into());
            let base_offset = 10_usize.pow(precision.into());
            let power = precision as usize + (index - base_offset) / (9 * shift);
            let power_offset = (0.9
                * (10_usize.pow(precision.into()) * (power - precision as usize)) as f64)
                as usize;
            let value = (index + shift - base_offset - power_offset) as u8
                * 10_u8.pow((power - precision as usize + 1) as u32);
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
