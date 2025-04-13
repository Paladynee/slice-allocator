use core::marker::PhantomData;

pub struct AlignedRawSlice<'a, T> {
    slice: *const [T],
    _marker: PhantomData<&'a [T]>,
}

impl<'buf, T> AlignedRawSlice<'buf, T> {
    /// # Safety
    ///
    /// The pointed-to slice must be well-aligned for values of type T.
    pub const unsafe fn from_raw_slice(ptr: *const [T]) -> AlignedRawSlice<'buf, T> {
        AlignedRawSlice {
            slice: ptr,
            _marker: PhantomData,
        }
    }

    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn as_ptr(&self) -> *const T {
        self.slice as *const T
    }

    pub const fn as_raw_slice(&self) -> *const [T] {
        self.slice
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    pub const unsafe fn as_slice(&self) -> &[T] {
        unsafe { &*self.as_raw_slice() }
    }
}

pub struct AlignedMutRawSlice<'a, T> {
    slice: *mut [T],
    _marker: PhantomData<&'a [T]>,
}

impl<'buf, T> AlignedMutRawSlice<'buf, T> {
    /// # Safety
    ///
    /// The pointed-to slice must be well-aligned for values of type T,
    /// and valid for writes of ptr.len() consecutive values of type T.
    pub const unsafe fn from_mut_raw_slice(ptr: *mut [T]) -> AlignedMutRawSlice<'buf, T> {
        AlignedMutRawSlice {
            slice: ptr,
            _marker: PhantomData,
        }
    }

    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn as_ptr(&self) -> *const T {
        self.slice as *const T
    }

    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.slice as *mut T
    }

    pub const fn as_raw_slice(&self) -> *const [T] {
        self.slice
    }

    pub const fn as_mut_raw_slice(&mut self) -> *mut [T] {
        self.slice
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    pub const unsafe fn as_slice(&self) -> &[T] {
        unsafe { &*self.as_raw_slice() }
    }

    /// # Safety
    ///
    /// The entirety of the underlying allocation must be initialized.
    pub const unsafe fn as_mut_slice(&mut self) -> &[T] {
        unsafe { &*self.as_mut_raw_slice() }
    }
}
