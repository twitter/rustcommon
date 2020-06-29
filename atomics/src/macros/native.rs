// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! native {
    (
        $(#[$outer:meta])*
        pub struct $name:ident: $type:ty = $atomic:ty;
    ) => {
        $(#[$outer])*
        pub struct $name {
            inner: $atomic,
        }

        impl $name {
            #[inline]
            pub fn new(value: $type) -> $name {
                $name {
                    inner: <$atomic>::new(value),
                }
            }
        }

        impl Default for $name {
            fn default() -> $name {
                <$name>::new(<$type>::default())
            }
        }

        impl Atomic for $name {
            type Primitive = $type;


            #[inline]
            fn load(&self, ordering: Ordering) -> Self::Primitive {
                self.inner.load(ordering)
            }

            #[inline]
            fn store(&self, value: Self::Primitive, ordering: Ordering) {
                self.inner.store(value, ordering)
            }

            #[inline]
            fn swap(&self, new: Self::Primitive, ordering: Ordering) -> Self::Primitive {
                self.inner.swap(new, ordering)
            }

            #[inline]
            fn compare_and_swap(&self, current: Self::Primitive, new: Self::Primitive, ordering: Ordering) -> Self::Primitive {
                self.inner.compare_and_swap(current, new, ordering)
            }

            #[inline]
            fn compare_exchange(
                &self,
                current: Self::Primitive,
                new: Self::Primitive,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Self::Primitive, Self::Primitive> {
                self.inner.compare_exchange(current, new, success, failure)
            }

            #[inline]
            fn compare_exchange_weak(
                &self,
                current: Self::Primitive,
                new: Self::Primitive,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Self::Primitive, Self::Primitive> {
                self.inner
                    .compare_exchange_weak(current, new, success, failure)
            }
        }
    };
}
