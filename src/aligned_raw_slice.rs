use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{backing_alloc::BackingAllocation, unaligned_generic_buffer::UnalignedGenericBuffer};

pub struct AlignedRawSlice<'a, T> {
    slice: *const [T],
    _marker: PhantomData<&'a [T]>,
}

impl<T> AlignedRawSlice<'_, T> {
    /// # Safety
    ///
    /// The pointed-to slice must be non-null and well-aligned for values of type T.
    #[inline]
    pub const unsafe fn from_raw_slice(ptr: *const [T]) -> Self {
        AlignedRawSlice {
            slice: ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.slice.cast::<T>()
    }

    #[inline]
    #[must_use]
    pub const fn as_raw_slice(&self) -> *const [T] {
        self.slice
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    #[inline]
    #[must_use]
    pub const unsafe fn as_slice(&self) -> &[T] {
        unsafe { &*self.as_raw_slice() }
    }
}

pub struct AlignedMutRawSlice<'buf, T> {
    slice: *mut [T],
    _marker: PhantomData<&'buf [T]>,
}

impl<'buf, T> AlignedMutRawSlice<'buf, T> {
    /// # Safety
    ///
    /// The pointed-to slice must be non-null, well-aligned for values of type T,
    /// and valid for writes of `ptr.len()` consecutive values of type T.
    #[inline]
    pub const unsafe fn from_mut_raw_slice(ptr: *mut [T]) -> Self {
        AlignedMutRawSlice {
            slice: ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_unique_slice(slice: &'buf mut [u8]) -> Self {
        let mut ugb = UnalignedGenericBuffer::from_unique_slice(slice);
        ugb.as_mut_aligned_raw_slice()
    }

    #[inline]
    #[must_use]
    pub fn from_unique_uninit_slice(slice: &'buf mut [MaybeUninit<u8>]) -> Self {
        let mut ugb = UnalignedGenericBuffer::from_unique_uninit_slice(slice);
        ugb.as_mut_aligned_raw_slice()
    }

    #[inline]
    #[must_use]
    pub fn from_backing_allocation(mem: BackingAllocation<'buf>) -> Self {
        let mut ugb = UnalignedGenericBuffer::from_backing_allocation(mem);
        ugb.as_mut_aligned_raw_slice()
    }

    #[inline]
    #[must_use]
    pub fn from_unaligned_generic_buffer(mut ugb: UnalignedGenericBuffer<'buf, T>) -> Self {
        ugb.as_mut_aligned_raw_slice()
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.slice.cast_const().cast::<T>()
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.slice.cast::<T>()
    }

    #[inline]
    #[must_use]
    pub const fn as_raw_slice(&self) -> *const [T] {
        self.slice.cast_const()
    }

    #[inline]
    pub const fn as_mut_raw_slice(&mut self) -> *mut [T] {
        self.slice
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    #[inline]
    #[must_use]
    pub const unsafe fn as_slice(&self) -> &[T] {
        unsafe { &*self.as_raw_slice() }
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    #[inline]
    pub const unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { &mut *self.as_mut_raw_slice() }
    }
}
