// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! float_arithmetic {
    ($name:ident, $type:ty) => {
        impl Arithmetic for $name {
            #[inline]
            fn fetch_add(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                let load_ordering = match ordering {
                    Ordering::AcqRel => Ordering::Acquire,
                    Ordering::Release => Ordering::Relaxed,
                    _ => ordering,
                };
                let current = self.inner.load(load_ordering);
                let mut new = <$type>::from_bits(current) + value;
                loop {
                    let result = self.inner.compare_exchange(
                        current,
                        new.to_bits(),
                        ordering,
                        load_ordering,
                    );
                    match result {
                        Ok(v) => {
                            return <$type>::from_bits(v);
                        }
                        Err(v) => {
                            new = <$type>::from_bits(v) + value;
                        }
                    }
                }
            }

            #[inline]
            fn fetch_sub(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                let load_ordering = match ordering {
                    Ordering::AcqRel => Ordering::Acquire,
                    Ordering::Release => Ordering::Relaxed,
                    _ => ordering,
                };
                let current = self.inner.load(load_ordering);
                let mut new = <$type>::from_bits(current) - value;
                loop {
                    let result = self.inner.compare_exchange(
                        current,
                        new.to_bits(),
                        ordering,
                        load_ordering,
                    );
                    match result {
                        Ok(v) => {
                            return <$type>::from_bits(v);
                        }
                        Err(v) => {
                            new = <$type>::from_bits(v) - value;
                        }
                    }
                }
            }
        }
    };
}
