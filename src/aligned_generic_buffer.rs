use crate::aligned_raw_slice::AlignedMutRawSlice;
use core::marker::PhantomData;

#[repr(transparent)]
pub struct AlignedGenericBuffer<'buf, T> {
    mem: AlignedMutRawSlice<'buf, T>,
    _marker: PhantomData<&'buf mut T>,
}

impl<'buf, T> AlignedGenericBuffer<'buf, T> {
    pub const fn from_aligned_mut_raw_slice(mem: AlignedMutRawSlice<'buf, T>) -> AlignedGenericBuffer<'buf, T> {
        AlignedGenericBuffer { mem, _marker: PhantomData }
    }

    pub const fn len(&self) -> usize {
        self.mem.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn as_ptr(&self) -> *const T {
        self.mem.as_ptr()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.mem.as_mut_ptr()
    }

    pub const fn as_raw_slice(&self) -> *const [T] {
        self.mem.as_raw_slice()
    }

    pub const fn as_mut_raw_slice(&mut self) -> *mut [T] {
        self.mem.as_mut_raw_slice()
    }
}
