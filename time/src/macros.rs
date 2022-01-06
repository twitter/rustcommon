// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICEN

#[macro_export]
macro_rules! atomic {
    ($name:ident<$atomic:ty>, $type:ty) => {
        impl $name<$atomic> {
            pub fn new(value: $name<$type>) -> Self {
                Self {
                    inner: <$atomic>::new(value.inner),
                }
            }
            pub fn load(&self, ordering: Ordering) -> $name<$type> {
                $name {
                    inner: self.inner.load(ordering),
                }
            }
            pub fn store(&self, value: $name<$type>, ordering: Ordering) {
                self.inner.store(value.inner, ordering)
            }
            pub fn swap(&self, value: $name<$type>, ordering: Ordering) -> $name<$type> {
                $name {
                    inner: self.inner.swap(value.inner, ordering),
                }
            }
            pub fn compare_exchange(
                &self,
                current: $name<$type>,
                new: $name<$type>,
                success: Ordering,
                failure: Ordering,
            ) -> Result<$name<$type>, $name<$type>> {
                self.inner
                    .compare_exchange(current.inner, new.inner, success, failure)
                    .map_err(|e| $name { inner: e })
                    .map(|v| $name { inner: v })
            }
            pub fn compare_exchange_weak(
                &self,
                current: $name<$type>,
                new: $name<$type>,
                success: Ordering,
                failure: Ordering,
            ) -> Result<$name<$type>, $name<$type>> {
                self.inner
                    .compare_exchange_weak(current.inner, new.inner, success, failure)
                    .map_err(|e| $name { inner: e })
                    .map(|v| $name { inner: v })
            }
        }
    };
}

#[macro_export]
macro_rules! atomic_arithmetic {
    ($name:ident<$atomic:ty>, $value:ident<$type:ty>) => {
        impl $name<$atomic> {
            pub fn fetch_add(&self, value: $value<$type>, ordering: Ordering) -> $name<$type> {
                $name {
                    inner: self.inner.fetch_add(value.inner, ordering),
                }
            }
            pub fn fetch_sub(&self, value: $value<$type>, ordering: Ordering) -> $name<$type> {
                $name {
                    inner: self.inner.fetch_add(value.inner, ordering),
                }
            }
        }
    }
}

#[macro_export]
macro_rules! unit {
    ($name:ident<$type:ty>) => {
        impl $name<$type> {
            pub fn checked_add(&self, other: Self) -> Option<Self> {
                Some(Self {
                    inner: self.inner.checked_add(other.inner)?,
                })
            }

            pub fn checked_sub(&self, other: Self) -> Option<Self> {
                Some(Self {
                    inner: self.inner.checked_sub(other.inner)?,
                })
            }

            pub fn saturating_add(&self, other: Self) -> Self {
                Self {
                    inner: self.inner.saturating_add(other.inner),
                }
            }

            pub fn saturating_sub(&self, other: Self) -> Self {
                Self {
                    inner: self.inner.saturating_sub(other.inner),
                }
            }
        }

        impl core::ops::Add<$name<$type>> for $name<$type> {
            type Output = Self;

            fn add(self, other: Self) -> Self::Output {
                Self::Output {
                    inner: self.inner + other.inner,
                }
            }
        }

        impl core::ops::AddAssign<$name<$type>> for $name<$type> {
            fn add_assign(&mut self, other: Self) {
                self.inner += other.inner;
            }
        }

        impl core::ops::Sub<$name<$type>> for $name<$type> {
            type Output = Self;

            fn sub(self, other: Self) -> Self::Output {
                Self::Output {
                    inner: self.inner - other.inner,
                }
            }
        }

        impl core::ops::SubAssign<$name<$type>> for $name<$type> {
            fn sub_assign(&mut self, other: Self) {
                self.inner -= other.inner;
            }
        }
    };
}

#[macro_export]
macro_rules! instant {
    ($name:ident<$unit:ty>) => {
        impl $name<$unit> {
            pub fn elapsed(&self) -> Duration<$unit> {
                let now = Self::now();
                now - *self
            }

            pub fn duration_since(&self, earlier: Self) -> Duration<$unit> {
                Duration {
                    inner: self
                        .inner
                        .checked_sub(earlier.inner)
                        .expect("supplied instant is later than self"),
                }
            }

            pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration<$unit>> {
                Some(Duration {
                    inner: self.inner.checked_sub(earlier.inner)?,
                })
            }

            pub fn saturating_duration_since(&self, earlier: Self) -> Duration<$unit> {
                Duration {
                    inner: self.inner.saturating_sub(earlier.inner),
                }
            }

            pub fn checked_add(&self, other: Duration<$unit>) -> Option<Self> {
                Some(Self {
                    inner: self.inner.checked_add(other.inner)?,
                })
            }

            pub fn checked_sub(&self, other: Duration<$unit>) -> Option<Self> {
                Some(Self {
                    inner: self.inner.checked_sub(other.inner)?,
                })
            }
        }

        impl core::ops::Add<Duration<$unit>> for $name<$unit> {
            type Output = Self;

            fn add(self, other: Duration<$unit>) -> Self::Output {
                Self::Output {
                    inner: self.inner + other.inner,
                }
            }
        }

        impl core::ops::Sub<Duration<$unit>> for $name<$unit> {
            type Output = Self;

            fn sub(self, other: Duration<$unit>) -> Self::Output {
                Self::Output {
                    inner: self.inner - other.inner,
                }
            }
        }

        impl core::ops::AddAssign<Duration<$unit>> for $name<$unit> {
            fn add_assign(&mut self, other: Duration<$unit>) {
                self.inner += other.inner;
            }
        }

        impl core::ops::Add<$name<$unit>> for $name<$unit> {
            type Output = Duration<$unit>;

            fn add(self, other: Self) -> Self::Output {
                Self::Output {
                    inner: self.inner + other.inner,
                }
            }
        }

        impl core::ops::SubAssign<Duration<$unit>> for $name<$unit> {
            fn sub_assign(&mut self, other: Duration<$unit>) {
                self.inner -= other.inner;
            }
        }

        impl core::ops::Sub<$name<$unit>> for $name<$unit> {
            type Output = Duration<$unit>;

            fn sub(self, other: Self) -> Self::Output {
                self.duration_since(other)
            }
        }
    };
}
