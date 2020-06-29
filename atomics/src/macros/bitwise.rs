// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! bitwise {
    ($name:ident, $type:ty) => {
        impl Bitwise for $name {
            #[inline]
            fn fetch_and(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                self.inner.fetch_and(value, ordering)
            }

            #[inline]
            fn fetch_nand(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                self.inner.fetch_nand(value, ordering)
            }

            #[inline]
            fn fetch_or(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                self.inner.fetch_or(value, ordering)
            }

            #[inline]
            fn fetch_xor(
                &self,
                value: <Self as Atomic>::Primitive,
                ordering: Ordering,
            ) -> <Self as Atomic>::Primitive {
                self.inner.fetch_xor(value, ordering)
            }
        }
    };
}
