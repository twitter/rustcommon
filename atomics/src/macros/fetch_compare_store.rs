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
                self.inner.fetch_max(value, ordering)
            }

            fn fetch_min(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                self.inner.fetch_min(value, ordering)
            }
        }
    };
}
