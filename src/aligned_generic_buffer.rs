use crate::aligned_raw_slice::AlignedMutRawSlice;
use core::marker::PhantomData;

#[repr(transparent)]
pub struct AlignedGenericBuffer<'buf, T> {
    mem: AlignedMutRawSlice<'buf, T>,
    _marker: PhantomData<&'buf mut T>,
}

impl<'buf, T> AlignedGenericBuffer<'buf, T> {
    #[inline]
    #[must_use]
    pub const fn from_aligned_mut_raw_slice(mem: AlignedMutRawSlice<'buf, T>) -> Self {
        Self { mem, _marker: PhantomData }
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.mem.len()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.mem.as_ptr()
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.mem.as_mut_ptr()
    }

    #[inline]
    #[must_use]
    pub const fn as_raw_slice(&self) -> *const [T] {
        self.mem.as_raw_slice()
    }

    #[inline]
    pub const fn as_mut_raw_slice(&mut self) -> *mut [T] {
        self.mem.as_mut_raw_slice()
    }
}
