use crate::align_twiddle::next_aligned_addr;
use crate::aligned_raw_slice::AlignedMutRawSlice;
use crate::aligned_raw_slice::AlignedRawSlice;
use crate::backing_alloc::BackingAllocation;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr; // Add this import for align_of and size_of

#[repr(transparent)]
pub struct UnalignedGenericBuffer<'buf, T> {
    mem: BackingAllocation<'buf>,
    _marker: PhantomData<&'buf mut T>,
}

impl<'buf, T> UnalignedGenericBuffer<'buf, T> {
    pub const fn from_backing_allocation(mem: BackingAllocation<'buf>) -> UnalignedGenericBuffer<'buf, T> {
        UnalignedGenericBuffer { mem, _marker: PhantomData }
    }

    /// Returns the amount of T's the underlying buffer can meaningfully fit
    /// without overflowing.
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

    /// Calculates which address after [`as_unaligned_ptr`](Self::as_unaligned_ptr)
    /// is valid for a `T` to exist. The returned pointer may not be sound to dereference.
    pub fn as_next_aligned_ptr(&self) -> *const T {
        self.as_unaligned_ptr().map_addr(|addr| next_aligned_addr(addr, align_of::<T>()))
    }

    /// Calculates which address after [`as_unaligned_ptr`](Self::as_unaligned_ptr)
    /// is valid for a `T` to exist. The returned pointer may not be sound to dereference.
    pub fn as_next_aligned_mut_ptr(&mut self) -> *mut T {
        self.as_unaligned_mut_ptr().map_addr(|addr| next_aligned_addr(addr, align_of::<T>()))
    }

    pub const fn as_unaligned_slice(&self) -> *const [T] {
        self.mem.as_raw_slice() as *const [T]
    }

    pub const fn as_unaligned_ptr(&self) -> *const T {
        self.mem.as_ptr() as *const T
    }

    pub const fn as_unaligned_mut_slice(&mut self) -> *mut [T] {
        self.mem.as_mut_raw_slice() as *mut [T]
    }

    pub const fn as_unaligned_mut_ptr(&mut self) -> *mut T {
        self.mem.as_mut_ptr() as *mut T
    }

    pub fn as_raw_slice(&self) -> *const [T] {
        let data = self.as_next_aligned_ptr();
        let len = self.valid_len();

        ptr::slice_from_raw_parts(data, len)
    }

    pub fn as_mut_raw_slice(&mut self) -> *mut [T] {
        let data = self.as_next_aligned_mut_ptr();
        let len = self.valid_len();

        ptr::slice_from_raw_parts_mut(data, len)
    }

    pub fn as_aligned_raw_slice(&self) -> AlignedRawSlice<'buf, T> {
        // Safety: as_raw_slice returns an aligned immutable slice.
        unsafe { AlignedRawSlice::from_raw_slice(self.as_raw_slice()) }
    }

    pub fn as_mut_aligned_raw_slice(&mut self) -> AlignedMutRawSlice<'buf, T> {
        // Safety: as_raw_mut_slice returns an aligned mutable slice.
        unsafe { AlignedMutRawSlice::from_mut_raw_slice(self.as_mut_raw_slice()) }
    }
}
