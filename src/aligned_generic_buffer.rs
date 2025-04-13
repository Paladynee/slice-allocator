use crate::aligned_raw_slice::AlignedMutRawSlice;
use crate::backing_alloc::BackingAllocation;
use crate::unaligned_generic_buffer::UnalignedGenericBuffer;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

#[repr(transparent)]
pub struct AlignedGenericBuffer<'buf, T> {
    mem: AlignedMutRawSlice<'buf, T>,
    _marker: PhantomData<&'buf mut T>,
}

impl<'buf, T> AlignedGenericBuffer<'buf, T> {
    #[inline]
    #[must_use]
    pub fn from_unique_slice(slice: &'buf mut [u8]) -> Self {
        let mem = AlignedMutRawSlice::from_unique_slice(slice);
        AlignedGenericBuffer { mem, _marker: PhantomData }
    }

    #[inline]
    #[must_use]
    pub fn from_unique_uninit_slice(slice: &'buf mut [MaybeUninit<u8>]) -> Self {
        let mem = AlignedMutRawSlice::from_unique_uninit_slice(slice);
        AlignedGenericBuffer { mem, _marker: PhantomData }
    }

    #[inline]
    #[must_use]
    pub fn from_backing_allocation(mem: BackingAllocation<'buf>) -> Self {
        let mem = AlignedMutRawSlice::from_backing_allocation(mem);
        AlignedGenericBuffer { mem, _marker: PhantomData }
    }

    #[inline]
    #[must_use]
    pub fn from_unaligned_generic_buffer(mut ugb: UnalignedGenericBuffer<'buf, T>) -> Self {
        let mem = ugb.as_mut_aligned_raw_slice();
        AlignedGenericBuffer { mem, _marker: PhantomData }
    }

    #[inline]
    #[must_use]
    pub const fn from_aligned_mut_raw_slice(mem: AlignedMutRawSlice<'buf, T>) -> Self {
        AlignedGenericBuffer { mem, _marker: PhantomData }
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.mem.len()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.mem.is_empty()
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
