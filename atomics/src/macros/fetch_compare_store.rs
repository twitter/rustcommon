// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! fetch_compare_store {
    ($name:ident, $type:ty) => {
        impl FetchCompareStore for $name {
            fn fetch_max(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                let load_ordering = match ordering {
                    Ordering::AcqRel => Ordering::Acquire,
                    Ordering::Release => Ordering::Relaxed,
                    _ => ordering,
                };
                let mut previous = self.load(load_ordering);
                if value <= previous {
                    // new value is not larger, return previous
                    previous
                } else {
                    loop {
                        let result = self.compare_and_swap(previous, value, ordering);
                        if result == previous {
                            // updated successfully. return previous value.
                            return previous;
                        }
                        previous = result;
                        if previous >= value {
                            // value concurrently modified and now new value is not
                            // larger. return updated previous value.
                            return previous;
                        }
                    }
                }
            }

            fn fetch_min(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                let load_ordering = match ordering {
                    Ordering::AcqRel => Ordering::Acquire,
                    Ordering::Release => Ordering::Relaxed,
                    _ => ordering,
                };
                let mut previous = self.load(load_ordering);
                if value >= previous {
                    // new value is not smaller, return previous value.
                    previous
                } else {
                    loop {
                        let result = self.compare_and_swap(previous, value, ordering);
                        if result == previous {
                            // updated successfully. return previous value.
                            return previous;
                        }
                        previous = result;
                        if previous <= value {
                            // value concurrently modified and now new value is not
                            // smaller. return updated previous value.
                            return previous;
                        }
                    }
                }
            }
        }
    };
}
