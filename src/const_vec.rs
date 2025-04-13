use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem;
use core::ptr;
use core::ptr::NonNull;

use crate::unaligned_const_allocator::UnalignedConstStackAllocator;

pub struct ConstRawVec<'alloc, T> {
    ptr: NonNull<T>,
    cap: usize,
    _alloc_in: PhantomData<UnalignedConstStackAllocator<'alloc>>,
}

impl<'alloc, T> ConstRawVec<'alloc, T> {
    #[inline]
    pub const fn new_const_in(_alloc: &mut UnalignedConstStackAllocator<'alloc>) -> Self {
        assert!(mem::size_of::<T>() != 0, "ZST currently unsupported but possible to implement");
        ConstRawVec {
            ptr: NonNull::dangling(),
            cap: 0,
            _alloc_in: PhantomData,
        }
    }

    #[inline]
    pub const fn with_capacity_const_in(cap: usize, allocator: &mut UnalignedConstStackAllocator<'alloc>) -> Self {
        assert!(mem::size_of::<T>() != 0, "ZST currently unsupported but possible to implement");
        let Ok(layout) = Layout::array::<T>(cap) else {
            panic!("allocation failed, handle_alloc_error is not yet stable in const")
        };

        let ptr = allocator.alloc_const_unaligned(layout).cast();

        ConstRawVec {
            ptr,
            cap,
            _alloc_in: PhantomData,
        }
    }

    #[inline]
    pub const fn grow_const(&mut self, allocator: &mut UnalignedConstStackAllocator<'alloc>) {
        let next_cap = if self.cap == 0 { 1 } else { self.cap * 2 };
        let Ok(new_layout) = Layout::array::<T>(next_cap) else {
            panic!("allocation failed, handle_alloc_error is not yet stable in const")
        };

        let new_ptr = if self.cap == 0 {
            allocator.alloc_const_unaligned(new_layout)
        } else {
            let Ok(old_layout) = Layout::array::<T>(self.cap) else {
                panic!("allocation failed, handle_alloc_error is not yet stable in const")
            };
            unsafe { allocator.realloc_const_unaligned(self.ptr.cast(), old_layout, new_layout) }
        };

        self.ptr = new_ptr.cast();
        self.cap = next_cap;
    }

    #[inline]
    pub const fn drop(self, alloc: &mut UnalignedConstStackAllocator<'alloc>) {
        let Ok(layout) = Layout::array::<T>(self.cap) else {
            panic!("deallocation failed, handle_alloc_error is not yet stable in const")
        };

        // deallocate the backing buffer
        unsafe { alloc.dealloc_const_unaligned(self.ptr.cast(), layout) };
    }
}

pub struct ConstVec<'alloc, T> {
    buf: ConstRawVec<'alloc, T>,
    len: usize,
    _alloc_in: PhantomData<UnalignedConstStackAllocator<'alloc>>,
}

impl<'alloc, T> ConstVec<'alloc, T> {
    #[inline]
    pub const fn new_const(alloc: &mut UnalignedConstStackAllocator<'alloc>) -> Self {
        ConstVec {
            buf: ConstRawVec::new_const_in(alloc),
            len: 0,
            _alloc_in: PhantomData,
        }
    }

    #[inline]
    pub const fn with_capacity_const_in(cap: usize, alloc: &mut UnalignedConstStackAllocator<'alloc>) -> Self {
        ConstVec {
            buf: ConstRawVec::with_capacity_const_in(cap, alloc),
            len: 0,
            _alloc_in: PhantomData,
        }
    }

    #[inline]
    const fn grow_const(&mut self, allocator: &mut UnalignedConstStackAllocator<'alloc>) {
        self.buf.grow_const(allocator);
    }

    #[inline]
    const fn needs_grow(&self) -> bool {
        self.len >= self.buf.cap
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn push_const(&mut self, allocator: &mut UnalignedConstStackAllocator<'alloc>, value: T) {
        if self.needs_grow() {
            self.grow_const(allocator);
        }

        let ptr = unsafe { self.buf.ptr.as_ptr().add(self.len) };
        unsafe {
            ptr::write_unaligned(ptr, value);
            self.len += 1;
        }
    }

    #[inline]
    pub const fn pop_const(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;

        let ptr = unsafe { self.buf.ptr.as_ptr().add(self.len) };
        unsafe {
            let value = ptr::read_unaligned(ptr);
            Some(value)
        }
    }

    #[inline]
    pub const fn drop(self, alloc: &mut UnalignedConstStackAllocator<'alloc>) {
        // we cannot drop unaligned items in-place.

        // we cannot evaluate the Drop implementation of T in const either.
        // otherwise we would drop contained elements by popping instead:
        // while let Some(_value) = self.pop_const() {}

        // deallocation is done by ConstRawVec
        self.buf.drop(alloc);
    }
}

#[test]
fn const_vec_test() {
    test()
}

#[inline]
pub const fn test() {
    let mut memory = [0u8; 1024];
    let mut allocator = UnalignedConstStackAllocator::from_unique_slice(&mut memory);

    let mut constvec = ConstVec::<u32>::new_const(&mut allocator);

    constvec.push_const(&mut allocator, 1);
    constvec.push_const(&mut allocator, 2);
    constvec.push_const(&mut allocator, 3);
    constvec.push_const(&mut allocator, 4);

    let mut another_const_vec = ConstVec::<u64>::with_capacity_const_in(4, &mut allocator);
    another_const_vec.push_const(&mut allocator, 1);
    another_const_vec.push_const(&mut allocator, 2);

    assert!(constvec.len() == 4);
    match constvec.pop_const() {
        Some(value) => assert!(value == 4),
        None => panic!("pop failed"),
    }
    assert!(constvec.len() == 3);
    match constvec.pop_const() {
        Some(value) => assert!(value == 3),
        None => panic!("pop failed"),
    }
    assert!(constvec.len() == 2);
    match constvec.pop_const() {
        Some(value) => assert!(value == 2),
        None => panic!("pop failed"),
    }
    assert!(constvec.len() == 1);
    match constvec.pop_const() {
        Some(value) => assert!(value == 1),
        None => panic!("pop failed"),
    }
    assert!(constvec.is_empty());
    if let Some(_value) = constvec.pop_const() {
        panic!("pop should not return a value");
    }

    constvec.drop(&mut allocator);
}
