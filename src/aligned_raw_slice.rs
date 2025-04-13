use core::marker::PhantomData;

pub struct AlignedRawSlice<'a, T> {
    slice: *const [T],
    _marker: PhantomData<&'a [T]>,
}

impl<T> AlignedRawSlice<'_, T> {
    /// # Safety
    ///
    /// The pointed-to slice must be well-aligned for values of type T.
    #[inline]
    pub const unsafe fn from_raw_slice(ptr: *const [T]) -> Self {
        Self {
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
        self.len() == 0
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

pub struct AlignedMutRawSlice<'a, T> {
    slice: *mut [T],
    _marker: PhantomData<&'a [T]>,
}

impl<T> AlignedMutRawSlice<'_, T> {
    /// # Safety
    ///
    /// The pointed-to slice must be well-aligned for values of type T,
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
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.slice as *const T
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.slice.cast::<T>()
    }

    #[inline]
    #[must_use]
    pub const fn as_raw_slice(&self) -> *const [T] {
        self.slice
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
    pub const unsafe fn as_mut_slice(&mut self) -> &[T] {
        unsafe { &*self.as_mut_raw_slice() }
    }
}
