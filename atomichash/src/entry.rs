use core::sync::atomic::*;
use std::borrow::Borrow;
use std::sync::Arc;

pub(crate) struct RawEntry<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
}

impl<K, V> RawEntry<K, V> {
    pub(crate) fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

pub(crate) struct Entry<K, V> {
    ptr: AtomicPtr<Arc<RawEntry<K, V>>>,
}

impl<K, V> Entry<K, V> {
    pub(crate) fn empty() -> Self {
        Self {
            ptr: AtomicPtr::new(core::ptr::null_mut() as *mut Arc<RawEntry<K, V>>),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ptr.load(Ordering::Relaxed).is_null()
    }

    pub(crate) fn clear(&self) {
        self.ptr.store(
            core::ptr::null_mut() as *mut Arc<RawEntry<K, V>>,
            Ordering::Relaxed,
        );
    }

    pub(crate) fn swap(&self, raw: RawEntry<K, V>) -> Option<RawEntry<K, V>> {
        let ptr = Box::into_raw(Box::new(Arc::new(raw)));
        let old = self.ptr.swap(ptr, Ordering::Relaxed);
        if !old.is_null() {
            unsafe {
                match Arc::try_unwrap(*(Box::from_raw(old))) {
                    Ok(raw) => Some(raw),
                    Err(_) => None, // couldn't take, arc may have too many strong references
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn load(&self) -> Option<&RawEntry<K, V>> {
        let ptr = self.ptr.load(Ordering::Relaxed);
        if ptr.is_null() {
            None
        } else {
            unsafe { Some((*ptr).borrow() as &RawEntry<K, V>) }
        }
    }
}
