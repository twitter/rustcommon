mod u16;
mod u32;
mod u64;
mod u8;

pub trait Indexing
where
    Self: Sized + Copy,
{
    fn get_index(value: Self, max: Self, exact: Self, precision: u8) -> Result<usize, ()>;
    fn get_min_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()>;
    fn get_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()>;

    fn get_max_value(
        index: usize,
        buckets: usize,
        max: Self,
        exact: Self,
        precision: u8,
    ) -> Result<Self, ()>;

    fn constrain_precision(precision: u8) -> u8;
    fn constrain_exact(max: Self, precision: u8) -> Self;
}
