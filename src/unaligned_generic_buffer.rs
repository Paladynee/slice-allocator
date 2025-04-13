use crate::aligned_raw_slice::AlignedMutRawSlice;
use crate::aligned_raw_slice::AlignedRawSlice;
use crate::backing_alloc::BackingAllocation;
use crate::const_allocator_shared::next_aligned_addr;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::mem::{align_of, size_of};
use core::ptr;

#[repr(transparent)]
pub struct UnalignedGenericBuffer<'buf, T> {
    mem: BackingAllocation<'buf>,
    _marker: PhantomData<&'buf mut T>,
}

impl<'buf, T> UnalignedGenericBuffer<'buf, T> {
    #[inline]
    #[must_use]
    pub const fn from_backing_allocation(mem: BackingAllocation<'buf>) -> Self {
        UnalignedGenericBuffer { mem, _marker: PhantomData }
    }

    #[inline]
    #[must_use]
    pub const fn from_unique_slice(slice: &'buf mut [u8]) -> Self {
        let backing_allocation = BackingAllocation::from_unique_slice(slice);
        UnalignedGenericBuffer {
            mem: backing_allocation,
            _marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_unique_uninit_slice(slice: &'buf mut [MaybeUninit<u8>]) -> Self {
        let backing_allocation = BackingAllocation::from_unique_uninit_slice(slice);
        UnalignedGenericBuffer {
            mem: backing_allocation,
            _marker: PhantomData,
        }
    }

    /// Returns the amount of T's the underlying buffer can meaningfully fit
    /// without overflowing.
    #[inline]
    #[must_use]
    pub fn valid_len(&self) -> usize {
        let buffer_size = self.mem.len();
        let aligned_start = next_aligned_addr(self.mem.as_ptr() as usize, align_of::<T>());
        let original_start = self.mem.as_ptr() as usize;

        // Calculate padding needed for alignment
        let alignment_padding = aligned_start.saturating_sub(original_start);

        // Calculate remaining space after alignment
        if alignment_padding >= buffer_size {
            return 0; // Not enough space even for alignment
        }

        let remaining_space = buffer_size - alignment_padding;

        // Calculate how many T's can fit in the remaining space
        remaining_space / size_of::<T>()
    }

    #[inline]
    #[must_use]
    pub const fn unaligned_len(&self) -> usize {
        self.mem.len() / size_of::<T>()
    }

    /// Calculates which address after [`as_unaligned_ptr`](Self::as_unaligned_ptr)
    /// is valid for a `T` to exist. The returned pointer may not be sound to dereference.
    #[inline]
    #[must_use]
    pub fn as_next_aligned_ptr(&self) -> *const T {
        self.as_unaligned_ptr().map_addr(|addr| next_aligned_addr(addr, align_of::<T>()))
    }

    /// Calculates which address after [`as_unaligned_ptr`](Self::as_unaligned_ptr)
    /// is valid for a `T` to exist. The returned pointer may not be sound to dereference.
    #[inline]
    pub fn as_next_aligned_mut_ptr(&mut self) -> *mut T {
        self.as_unaligned_mut_ptr().map_addr(|addr| next_aligned_addr(addr, align_of::<T>()))
    }

    #[inline]
    #[must_use]
    pub const fn as_unaligned_slice(&self) -> *const [T] {
        self.mem.as_raw_slice() as *const [T]
    }

    #[inline]
    #[must_use]
    pub const fn as_unaligned_ptr(&self) -> *const T {
        self.mem.as_ptr().cast::<T>()
    }

    #[inline]
    pub const fn as_unaligned_mut_slice(&mut self) -> *mut [T] {
        self.mem.as_mut_raw_slice() as *mut [T]
    }

    #[inline]
    pub const fn as_unaligned_mut_ptr(&mut self) -> *mut T {
        self.mem.as_mut_ptr().cast::<T>()
    }

    #[inline]
    #[must_use]
    pub fn as_raw_slice(&self) -> *const [T] {
        let data = self.as_next_aligned_ptr();
        let len = self.valid_len();

        ptr::slice_from_raw_parts(data, len)
    }

    #[inline]
    pub fn as_mut_raw_slice(&mut self) -> *mut [T] {
        let data = self.as_next_aligned_mut_ptr();
        let len = self.valid_len();

        ptr::slice_from_raw_parts_mut(data, len)
    }

    #[inline]
    #[must_use]
    pub fn as_aligned_raw_slice(&self) -> AlignedRawSlice<'buf, T> {
        // Safety: as_raw_slice returns an aligned immutable slice.
        unsafe { AlignedRawSlice::from_raw_slice(self.as_raw_slice()) }
    }

    #[inline]
    pub fn as_mut_aligned_raw_slice(&mut self) -> AlignedMutRawSlice<'buf, T> {
        // Safety: as_raw_mut_slice returns an aligned mutable slice.
        unsafe { AlignedMutRawSlice::from_mut_raw_slice(self.as_mut_raw_slice()) }
    }
}
