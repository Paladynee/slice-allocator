use core::mem::MaybeUninit;
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

    let data = ptr::from_ref::<[Src]>(src).cast::<Src>().cast::<Dst>();
    let len = src.len();

    ptr::slice_from_raw_parts(data, len)
}

/// This is private API.
///
/// The resulting pointer may not be safe to dereference.
pub(crate) const fn cast_slice_mut<Src, Dst>(src: &mut [Src]) -> *mut [Dst] {
    assert!(
        size_of::<Src>() == size_of::<Dst>(),
        "this function can't be used with types of different size"
    );

    let data = ptr::from_mut::<[Src]>(src).cast::<Src>().cast::<Dst>();
    let len = src.len();

    ptr::slice_from_raw_parts_mut(data, len)
}

impl<'buf> BackingAllocation<'buf> {
    #[inline]
    pub const fn from_unique_slice(slice: &'buf mut [u8]) -> Self {
        let uninit_slice: &'buf mut [MaybeUninit<u8>] = {
            // Safety: the reborrow comes from a concrete mutable slice,
            // and the lifetime is unchanged. Layout of MaybeUninit<T> is
            // guaranteed to be same with T.
            unsafe { &mut *(cast_slice_mut::<u8, MaybeUninit<u8>>(slice)) }
        };

        BackingAllocation::from_unique_uninit_slice(uninit_slice)
    }

    #[inline]
    pub const fn from_unique_uninit_slice(slice: &'buf mut [MaybeUninit<u8>]) -> Self {
        // overwrite the contents of the slice with 0xAA in debug builds
        if cfg!(debug_assertions) {
            let mut i = 0;
            while i < slice.len() {
                slice[i].write(0xAA);
                i += 1;
            }
        }

        BackingAllocation { slice }
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
    pub const fn as_ptr(&self) -> *const u8 {
        self.slice.as_ptr().cast()
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.slice.as_mut_ptr().cast()
    }

    #[inline]
    #[must_use]
    pub const fn as_raw_slice(&self) -> *const [u8] {
        cast_slice::<MaybeUninit<u8>, u8>(self.slice)
    }

    #[inline]
    pub const fn as_mut_raw_slice(&mut self) -> *mut [u8] {
        cast_slice_mut::<MaybeUninit<u8>, u8>(self.slice)
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    #[inline]
    #[must_use]
    pub const unsafe fn as_slice(&self) -> &[u8] {
        unsafe { &*self.as_raw_slice() }
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    #[inline]
    pub const unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { &mut *self.as_mut_raw_slice() }
    }

    #[inline]
    #[must_use]
    pub const fn into_inner(self) -> &'buf mut [MaybeUninit<u8>] {
        self.slice
    }
}
