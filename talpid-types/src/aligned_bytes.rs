use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut, Range},
};

/// A type that behaves lite a `Box<[u8]>`, but it ensures that the buffer is aligned to `A`.
pub struct AlignedBytes<A> {
    unaligned: Box<[u8]>,
    range: Range<usize>,
    _align_t: PhantomData<A>,
}

impl<A> AlignedBytes<A> {
    // Allocate a buffer of exactly `len` bytes, aligned to `T`.
    pub fn new(len: usize) -> Self {
        let alignment = align_of::<A>();
        let unaligned = vec![0u8; len + alignment].into_boxed_slice();
        let offset = unaligned.as_ptr().align_offset(alignment);
        let range = offset..(len - alignment + offset);
        Self {
            unaligned,
            range,
            _align_t: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.deref().as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.deref_mut().as_mut_ptr()
    }
}

impl<A> Deref for AlignedBytes<A> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.unaligned[self.range.clone()]
    }
}

impl<A> DerefMut for AlignedBytes<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unaligned[self.range.clone()]
    }
}
