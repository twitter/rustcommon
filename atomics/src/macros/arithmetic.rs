// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! arithmetic {
    ($name:ident, $type:ty) => {
        impl Arithmetic for $name {
            #[inline]
            fn fetch_add(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                self.inner.fetch_add(value, ordering)
            }

            #[inline]
            fn fetch_sub(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                self.inner.fetch_sub(value, ordering)
            }
        }
    };
}
