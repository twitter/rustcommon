// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! float {
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
                    inner: <$atomic>::new(value.to_bits()),
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
                <$type>::from_bits(self.inner.load(ordering))
            }

            #[inline]
            fn store(&self, value: Self::Primitive, ordering: Ordering) {
                self.inner.store(value.to_bits(), ordering)
            }

            #[inline]
            fn swap(&self, new: Self::Primitive, ordering: Ordering) -> Self::Primitive {
                <$type>::from_bits(self.inner.swap(new.to_bits(), ordering))
            }

            #[inline]
            fn compare_exchange(
                &self,
                current: Self::Primitive,
                new: Self::Primitive,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Self::Primitive, Self::Primitive> {
                self.inner
                    .compare_exchange(current.to_bits(), new.to_bits(), success, failure)
                    .map(<$type>::from_bits)
                    .map_err(<$type>::from_bits)
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
                    .compare_exchange_weak(current.to_bits(), new.to_bits(), success, failure)
                    .map(<$type>::from_bits)
                    .map_err(<$type>::from_bits)
            }
        }

        impl std::fmt::Debug for $name where $type: std::fmt::Debug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.inner)
            }
        }
    };
}
