// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// Based on the Rust Standard Library: https://github.com/rust-lang/rust
//
// Except as otherwise noted (below and/or in individual files), Rust is
// licensed under the Apache License, Version 2.0 <LICENSE-APACHE> or
// <http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT> or <http://opensource.org/licenses/MIT>, at your option.
//
// library/alloc/src/collections/linked_list.rs
//
// Based on commit: 9e779986aa2aaa6d28b48020f9da8f37b95959ee

// Based on Twitter ccommon: https://github.com/twitter/ccommon
//
// ccommon - a cache common library.
// Copyright (C) 2013 Twitter, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::cmp::Ordering;
use core::hash::Hash;
use core::hash::Hasher;
use core::iter::FusedIterator;
use core::iter::Iterator;
use core::marker::PhantomData;
use core::ptr::NonNull;

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    element: T,
}

/// A tail queue is headed by a pair of pointers, one to the head of the
/// list and the other to the tail of the list. The elements are doubly
/// linked so that an arbitrary element can be removed without a need to
/// traverse the list. New elements can be added to the list before or
/// after an existing element, at the head of the list, or at the end of
/// the list. A tail queue may be traversed in either direction.
///
/// This implementation is based on:
/// * The Rust Standard Library:
///   * https://github.com/rust-lang/rust
///   * Dual-licensed under Apache 2.0 / MIT license
/// * Twitter ccommon
///   * https://github.com/twitter/ccommon
///   * Licensed under Apache 2.0
///
/// See source for applicable copyright headers.
pub struct TailQ<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<Box<Node<T>>>,
}

impl<T> Default for TailQ<T> {
    /// Creates an empty `SList<T>`.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

pub struct IterMut<'a, T: 'a> {
    // Does *not* own exclusive access to the entire list. Be careful of aliased
    // pointers!
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

impl<T> Node<T> {
    fn new(element: T) -> Self {
        Node {
            next: None,
            prev: None,
            element,
        }
    }

    #[allow(clippy::boxed_local)]
    fn into_element(self: Box<Self>) -> T {
        self.element
    }
}

impl<T> TailQ<T> {
    /// Adds a node to the front of the list
    #[inline]
    fn push_front_node(&mut self, mut node: Box<Node<T>>) {
        node.next = self.head;
        let node = Some(Box::leak(node).into());
        if self.is_empty() {
            self.tail = node;
        } else {
            unsafe {
                (*self.head.unwrap().as_ptr()).prev = node;
            }
        }
        self.head = node;
        self.len += 1;
    }

    /// Adds a node to the back of the list
    #[inline]
    fn push_back_node(&mut self, mut node: Box<Node<T>>) {
        node.prev = self.tail;
        node.next = None;
        let node = Some(Box::leak(node).into());
        if self.is_empty() {
            self.head = node;
        } else {
            unsafe {
                (*self.tail.unwrap().as_ptr()).next = node;
            }
        }
        self.tail = node;
        self.len += 1;
    }

    /// Removes a node from the front of the list
    #[inline]
    fn pop_front_node(&mut self) -> Option<Box<Node<T>>> {
        // does not create mutable reference to whole node to keep aliasing
        // pointers valid.
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;
            if self.head.is_some() {
                (*self.head.unwrap().as_ptr()).prev = None;
            }
            self.len -= 1;
            if self.is_empty() {
                self.tail = None;
            }
            node
        })
    }

    /// Removes a node from the back of the list
    #[inline]
    fn pop_back_node(&mut self) -> Option<Box<Node<T>>> {
        // does not create mutable reference to whole node to keep aliasing
        // pointers valid.
        self.tail.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.tail = node.prev;
            if self.tail.is_some() {
                (*self.tail.unwrap().as_ptr()).next = None;
            }
            self.len -= 1;
            if self.is_empty() {
                self.head = None;
            }
            node
        })
    }
}

impl<T> TailQ<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn contains(&self, x: &T) -> bool
    where
        T: std::cmp::PartialEq,
    {
        self.iter().any(|e| e == x)
    }

    #[inline]
    pub fn front(&self) -> Option<&T> {
        unsafe { self.head.as_ref().map(|node| &node.as_ref().element) }
    }

    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.head.as_mut().map(|node| &mut node.as_mut().element) }
    }

    #[inline]
    pub fn back(&self) -> Option<&T> {
        unsafe { self.tail.as_ref().map(|node| &node.as_ref().element) }
    }

    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { self.tail.as_mut().map(|node| &mut node.as_mut().element) }
    }

    pub fn push_front(&mut self, element: T) {
        self.push_front_node(Box::new(Node::new(element)));
    }

    pub fn push_back(&mut self, element: T) {
        self.push_back_node(Box::new(Node::new(element)));
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(Node::into_element)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(Node::into_element)
    }

    pub fn split_off(&mut self, at: usize) -> TailQ<T> {
        let len = self.len();
        assert!(at <= len, "Cannot split off at a nonexistent index");

        // handle special cases if index is either start or end of list
        if at == 0 {
            return std::mem::take(self);
        } else if at == len {
            return Self::new();
        }

        let new_len = self.len - at;
        let new_tail = self.tail;

        // locate node starting from front or back
        let new_head = if at <= len / 2 {
            let mut node = self.head.unwrap();
            for _ in 0..at {
                unsafe {
                    node = (*node.as_ptr()).next.unwrap();
                }
            }
            node
        } else {
            let mut node = self.tail.unwrap();
            for _ in 0..len - at - 1 {
                unsafe {
                    node = (*node.as_ptr()).prev.unwrap();
                }
            }
            node
        };

        // remap pointers
        unsafe {
            self.tail = (*new_head.as_ptr()).prev;
            (*self.tail.as_ref().unwrap().as_ptr()).next = None;
            (*(new_head.as_ptr())).prev = None;
        }

        self.len = at;

        Self {
            head: Some(new_head),
            tail: new_tail,
            len: new_len,
            marker: self.marker,
        }
    }

    pub fn remove(&mut self, at: usize) -> T {
        let len = self.len();
        assert!(
            at < len,
            "Cannot remove at an index outside of the list bounds"
        );

        // special case for front or back
        if at == 0 {
            return self.pop_front().unwrap();
        } else if at == (self.len - 1) {
            return self.pop_back().unwrap();
        }

        // find node to remove, from front or from back depending on index
        // location
        let remove = if at <= len / 2 {
            let mut remove = self.head.unwrap();
            for _ in 0..at {
                unsafe {
                    remove = (*remove.as_ptr()).next.unwrap();
                }
            }
            remove
        } else {
            let mut remove = self.tail.unwrap();
            for _ in 0..len - at - 1 {
                unsafe {
                    remove = (*remove.as_ptr()).prev.unwrap();
                }
            }
            remove
        };

        // remap pointers to remove node from list
        unsafe {
            let remove = Box::from_raw(remove.as_ptr());
            if let Some(prev) = remove.prev {
                (*prev.as_ptr()).next = remove.next;
            }
            if let Some(next) = remove.next {
                (*next.as_ptr()).prev = remove.prev;
            }
            self.len -= 1;
            remove.element
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &*node.as_ptr();
                self.len -= 1;
                self.head = node.next;
                &node.element
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &*node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &node.element
            })
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.head = node.next;
                &mut node.element
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut T> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &mut node.element
            })
        }
    }
}

pub struct IntoIter<T> {
    list: TailQ<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.list.pop_front()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> IntoIterator for TailQ<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Consumes the list into an iterator yielding elements by value.
    #[inline]
    fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }
}

impl<'a, T> IntoIterator for &'a TailQ<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut TailQ<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T: PartialEq> PartialEq for TailQ<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
    }
}

impl<T: Eq> Eq for TailQ<T> {}

impl<T: PartialOrd> PartialOrd for TailQ<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for TailQ<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash> Hash for TailQ<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for elt in self {
            elt.hash(state);
        }
    }
}

unsafe impl<T: Send> Send for TailQ<T> {}
unsafe impl<T: Sync> Sync for TailQ<T> {}

unsafe impl<T: Sync> Send for Iter<'_, T> {}
unsafe impl<T: Sync> Sync for Iter<'_, T> {}

unsafe impl<T: Send> Send for IterMut<'_, T> {}
unsafe impl<T: Sync> Sync for IterMut<'_, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front() {
        let mut list = TailQ::<u8>::new();
        list.push_front(0);
        assert_eq!(list.front(), Some(&0));
        list.push_front(1);
        assert_eq!(list.front(), Some(&1));
        list.push_back(2);
        assert_eq!(list.front(), Some(&1));
        list.clear();
        assert_eq!(list.front(), None);
    }

    #[test]
    fn back() {
        let mut list = TailQ::<u8>::new();
        list.push_front(0);
        assert_eq!(list.back(), Some(&0));
        list.push_front(1);
        assert_eq!(list.back(), Some(&0));
        list.push_back(2);
        assert_eq!(list.back(), Some(&2));
        list.clear();
        assert_eq!(list.back(), None);
    }

    #[test]
    fn pop_front() {
        let mut list = TailQ::<u8>::new();
        list.push_front(0);
        assert_eq!(list.pop_front(), Some(0));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn push_front() {
        let mut list = TailQ::<usize>::new();
        for v in 0..100 {
            assert_eq!(list.len(), v);
            list.push_front(v);
            assert_eq!(list.len(), v + 1);
        }
        for v in (0..100).rev() {
            assert_eq!(list.len(), v + 1);
            assert_eq!(list.pop_front(), Some(v));
            assert_eq!(list.len(), v);
        }
    }

    #[test]
    fn push_back() {
        let mut list = TailQ::<usize>::new();
        list.push_back(0);
        assert_eq!(list.pop_front(), Some(0));
        assert_eq!(list.pop_front(), None);
        let items = 100;
        for v in 0..items {
            assert_eq!(list.len(), v);
            list.push_back(v);
            assert_eq!(list.len(), v + 1);
        }
        for v in 0..100 {
            assert_eq!(list.len(), items - v);
            assert_eq!(list.pop_front(), Some(v));
            assert_eq!(list.len(), items - (v + 1));
        }
    }

    #[test]
    fn remove() {
        let mut list = TailQ::<usize>::new();
        let items = 100;
        for start in 0..items {
            for v in 0..items {
                list.push_back(v);
            }
            for v in start..items {
                assert_eq!(list.remove(start), v);
            }
            list.clear();
        }
    }

    #[test]
    fn iter() {
        let mut list = TailQ::<usize>::new();
        let items = 100;
        for v in 0..items {
            list.push_back(v);
        }
        let mut iter = list.iter();
        for v in 0..items {
            assert_eq!(iter.next(), Some(&v));
        }
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn iter_reverse() {
        let mut list = TailQ::<usize>::new();
        let items = 100;
        for v in 0..items {
            list.push_back(v);
        }
        let mut iter = list.iter();
        for v in (0..items).rev() {
            assert_eq!(iter.next_back(), Some(&v));
        }
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn split_off() {
        let mut a = TailQ::<usize>::new();
        for v in 0..100 {
            a.push_back(v);
        }
        let b = a.split_off(50);
        assert_eq!(a.len(), 50);
        assert_eq!(b.len(), 50);
        assert_eq!(a.front(), Some(&0));
        assert_eq!(a.back(), Some(&49));
        assert_eq!(b.front(), Some(&50));
        assert_eq!(b.back(), Some(&99));
    }
}
