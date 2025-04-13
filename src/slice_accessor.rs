//! Man do I wish we had const traits.
#![allow(unused)]
#![allow(clippy::missing_safety_doc)]

pub trait RawSliceAccessor<T> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;
    fn as_raw_slice(&self) -> *const [T];
    fn as_raw_mut_slice(&mut self) -> *mut [T];
}

pub trait UnsafeSliceAccessor<T>: RawSliceAccessor<T> {
    unsafe fn as_slice(&self) -> &[T];
    unsafe fn as_mut_slice(&mut self) -> &mut [T];
}

pub trait SafeSliceAccessor<T>: RawSliceAccessor<T> {
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
}
