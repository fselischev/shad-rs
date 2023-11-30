#![no_std]

use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

pub struct ArrayVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, obj: T) -> Result<(), T> {
        if self.data.len() != self.len {
            self.data[self.len].write(obj);
            self.len += 1;
            Ok(())
        } else {
            Err(obj)
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.data[self.len].as_ptr().read() })
        } else {
            None
        }
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Index<usize> for ArrayVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.len {
            return unsafe { &*self.data[index].as_ptr() };
        }
        panic!("index out of bounds")
    }
}

impl<T, const N: usize> IndexMut<usize> for ArrayVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.len {
            return unsafe { &mut *self.data[index].as_mut_ptr() };
        }
        panic!("index out of bounds")
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        for elem in &mut self.data[0..self.len] {
            unsafe {
                elem.assume_init_drop();
            }
        }
    }
}
