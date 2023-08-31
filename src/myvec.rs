#![allow(unused)]

use core::slice;
use std::alloc::{alloc, handle_alloc_error, Layout, LayoutError};
use std::{marker::PhantomData, ptr::NonNull};

/// I'm mainly writing this type to understand Vector internals before trying to customize them.
pub struct MyVec<T> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
    cap: usize,
    len: usize,
}

impl<T> MyVec<T> {
    // Difficult to check size of T => We always use 4 while the stdlib chooses based on the type size.
    const MIN_NON_ZERO_CAP: usize = 8; // Use 8 because we're using u8 for fuzzing.

    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            _marker: PhantomData,
            cap: 0,
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut ret = Self {
            ptr: NonNull::dangling(),
            _marker: PhantomData,
            cap: 0,
            len: 0,
        };
        ret.allocate(capacity);
        ret
    }

    fn allocate(&mut self, capacity: usize) {
        if capacity == 0 {
        } else {
            let layout = match Layout::array::<T>(capacity) {
                Ok(layout) => layout,
                Err(_) => capacity_overflow(),
            };
            alloc_guard(layout.size());
            if layout.size() == 0 {
                // We have a Zero-Sized Type. There is no easy way to get this information
                // besides doing this (which might be efficient). For now Zero-Sized types are
                // not allowed in `MyVec`, use `std::vec::Vec` instead.
                panic!("Zero capacity")
            }
            let ptr = unsafe {
                // Undefined behavior if layout has a zero-size, we've handled this above
                alloc(layout)
            };
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            // This is safe because we've just handled null pointers.
            self.ptr = unsafe { NonNull::new_unchecked(ptr.cast::<T>()) };
            self.cap = capacity;
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn reserve(&mut self, additional: usize) {
        // Note: The stdlib does some interesting stuff to avoid inlining the grow code,
        // we're not doing that here.
        if self.needs_to_grow(additional) {
            self.grow_amortized(additional);
        }
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        if self.needs_to_grow(additional) {
            self.grow_exact(additional)
        }
    }

    fn grow_amortized(&mut self, additional: usize) {
        debug_assert!(additional > 0);
        let required_cap = match self.len.checked_add(additional) {
            Some(v) => v,
            None => capacity_overflow(),
        };

        // This guarantees exponential growth. The doubling cannot overflow
        // because `cap <= isize::MAX` and the type of `cap` is `usize`.

        let cap = (self.cap * 2).max(required_cap);
        let cap = Self::MIN_NON_ZERO_CAP.max(cap);

        self.resize_to(cap)
    }

    fn grow_exact(&mut self, additional: usize) {
        let cap = match self.len.checked_add(additional) {
            Some(v) => v,
            None => capacity_overflow(),
        };
        self.resize_to(cap);
    }

    fn resize_to(&mut self, cap: usize) {
        debug_assert!(self.len <= cap);
        if cap == 0 {
            self.dealloc()
        } else {
            let new_layout = Layout::array::<T>(cap);
            let ptr = finish_grow(new_layout, self.current_memory());
            self.ptr = ptr.cast();
            self.cap = cap;
        }
    }

    fn current_memory(&self) -> Option<NonNull<u8>> {
        if self.cap == 0 {
            None
        } else {
            Some(self.ptr.cast())
        }
    }

    fn needs_to_grow(&self, additional: usize) -> bool {
        additional > self.cap.wrapping_sub(self.len)
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow_amortized(1);
        }

        unsafe {
            let end = self.ptr.as_ptr().add(self.len);
            core::ptr::write(end, value);
            self.len += 1;
        }
    }

    pub fn shrink_to_fit(&mut self) {
        if self.cap > self.len {
            self.shrink(self.len);
        }
    }

    fn shrink(&mut self, cap: usize) {
        assert!(cap <= self.cap, "Tried to shrink to a larger capacity");

        if cap == self.cap {
            return;
        }

        self.resize_to(cap);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.cap > min_capacity {
            self.shrink(self.len.max(min_capacity));
        }
    }

    fn dealloc(&mut self) {
        // Call drop on all elements
        unsafe {
            core::ptr::drop_in_place(core::ptr::slice_from_raw_parts_mut(
                self.ptr.as_ptr(),
                self.len,
            ))
        }

        if let Some(ptr) = self.current_memory() {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe { std::alloc::dealloc(ptr.as_ptr(), layout) };
        }

        self.ptr = NonNull::dangling();
        self.cap = 0;
    }

    pub fn truncate(&mut self, len: usize) {
        if len > self.len {
            return;
        }
        unsafe {
            let remaining_len = self.len - len;
            let s = std::ptr::slice_from_raw_parts_mut(self.ptr.as_ptr().add(len), remaining_len);
            self.len = len;
            std::ptr::drop_in_place(s);
        }
    }
}

impl<T> std::ops::Deref for MyVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> core::ops::Drop for MyVec<T> {
    fn drop(&mut self) {
        self.dealloc();
    }
}

// Mostly taken from stdlin, apparently this is done separately to reduce compile times.
#[inline(never)]
fn finish_grow(
    new_layout: Result<Layout, LayoutError>,
    current_memory: Option<NonNull<u8>>,
) -> NonNull<u8> {
    let new_layout = match new_layout {
        Ok(v) => v,
        Err(_) => capacity_overflow(),
    };
    alloc_guard(new_layout.size());
    if new_layout.size() == 0 {
        panic!("Zero capacity")
    }

    let ptr = unsafe {
        match current_memory {
            Some(ptr) => std::alloc::realloc(ptr.as_ptr(), new_layout, new_layout.size()),
            None => std::alloc::alloc(new_layout),
        }
    };
    if ptr.is_null() {
        handle_alloc_error(new_layout);
    }
    // This is safe because we've just handled null pointers.
    unsafe { NonNull::new_unchecked(ptr) }
}

#[inline]
fn alloc_guard(alloc_size: usize) {
    if usize::BITS < 64 && alloc_size > isize::MAX as usize {
        capacity_overflow()
    }
}

fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}

// Note: We can't really check for zero-sized types, so we'll not do this optimization here.
// Not implemented: try_reserve, try_reserve_exact, shrink_to, into_boxed_slice,
//                  truncate, as_slice, as_mut_slice, as_ptr, as_mut_ptr, allocator, set_len,
//                  swap_remove, insert, remove, retain, retain_mut, dedup_by_key, dedup_by,
//                  push_with_capacity, pop, append, append_elements, drain, clear,
//                  len, is_empty, split_off, resize_with, leak, spare_capacity_mut, split_at_spare_mut,
//                  split_at_spare_mut_with_len, extend_with
// Not implemented (if T: Clone): resize, extend_from_slice, extend_from_within
// Not implemented (if [T; N]): into_flattened,
// Not implemented (if PartialEq): dedup
