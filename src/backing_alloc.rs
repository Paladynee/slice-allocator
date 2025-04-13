#![allow(unused)]
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::mem::align_of;
use core::mem::size_of;
use core::ptr;

#[repr(transparent)]
pub struct BackingAllocation<'buf> {
    slice: &'buf mut [MaybeUninit<u8>],
}

/// This is private API.
///
/// The resulting pointer may not be safe to dereference.
pub(crate) const fn cast_slice<Src, Dst>(src: &[Src]) -> *const [Dst] {
    assert!(
        size_of::<Src>() == size_of::<Dst>(),
        "this function can't be used with types of different size"
    );

    let data = src as *const [Src] as *const Src as *const Dst;
    let len = src.len();

    // Safety: unlike raw slices, slices are always non-null
    unsafe { ptr::slice_from_raw_parts(data, len) }
}

/// This is private API.
///
/// The resulting pointer may not be safe to dereference.
pub(crate) const fn cast_slice_mut<Src, Dst>(src: &mut [Src]) -> *mut [Dst] {
    assert!(
        size_of::<Src>() == size_of::<Dst>(),
        "this function can't be used with types of different size"
    );

    let data = src as *mut [Src] as *mut Src as *mut Dst;
    let len = src.len();

    // Safety: unlike raw slices, slices are always non-null
    unsafe { ptr::slice_from_raw_parts_mut(data, len) }
}

impl<'buf> BackingAllocation<'buf> {
    pub const fn from_unique_slice(slice: &'buf mut [u8]) -> BackingAllocation<'buf> {
        let uninit_slice: &'buf mut [MaybeUninit<u8>] = {
            // Safety: the reborrow comes from a concrete mutable slice,
            // and the lifetime is unchanged. Layout of MaybeUninit<T> is
            // guaranteed to be same with T.
            unsafe { &mut *(slice as *mut [u8] as *mut [MaybeUninit<u8>]) }
        };
        BackingAllocation { slice: uninit_slice }
    }

    pub const fn from_unique_uninit_slice(slice: &'buf mut [MaybeUninit<u8>]) -> BackingAllocation<'buf> {
        BackingAllocation { slice }
    }

    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.slice.as_ptr().cast()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.slice.as_mut_ptr().cast()
    }

    pub const fn as_raw_slice(&self) -> *const [u8] {
        cast_slice::<MaybeUninit<u8>, u8>(self.slice)
    }

    pub const fn as_mut_raw_slice(&mut self) -> *mut [u8] {
        cast_slice_mut::<MaybeUninit<u8>, u8>(self.slice)
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    pub const unsafe fn as_slice(&self) -> &[u8] {
        unsafe { &*self.as_raw_slice() }
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    pub const unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { &mut *self.as_mut_raw_slice() }
    }

    pub const fn into_inner(self) -> &'buf mut [MaybeUninit<u8>] {
        self.slice
    }
}
