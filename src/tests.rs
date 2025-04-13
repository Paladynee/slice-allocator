use crate::backing_alloc::BackingAllocation;
use crate::const_vec::ConstVec;
use crate::unaligned_const_allocator::UnalignedConstStackAllocator;
#[cfg(feature = "real_const_alloc")]
use core::intrinsics;
use core::marker::PhantomData;

#[test]
fn const_vec_test() {
    let data = const { const_vec() };
}

#[test]
fn runtime_comptime_interaction_test() {
    runtime_comptime_interaction();
}

#[test]
fn into_actual_const_allocated_test() {
    let slice = const { into_actual_const_allocated() };
    assert_eq!(slice.len(), 4);
    assert_eq!(slice[0], 1);
    assert_eq!(slice[1], 2);
    assert_eq!(slice[2], 3);
    assert_eq!(slice[3], 4);
}

#[cfg(feature = "real_const_alloc")]
#[test]
fn rust_intrinsic_const_allocate_can_escape_to_runtime_safely() {
    let data = const { something() };
    assert!(data.len() == 2);
    assert!(data[0] == 0xbbbbbbbb);
    assert!(data[1] == 0xaaaaaaaa);
}

#[inline]
const fn const_vec() {
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
    another_const_vec.drop(&mut allocator);
}

/// # Panics
///
/// this is a test.
#[inline]
fn runtime_comptime_interaction() {
    use alloc::vec;
    use alloc::vec::Vec;
    let mut rt_memory: Vec<u8> = vec![0; 1024];
    let mut alloc = UnalignedConstStackAllocator::from_unique_slice(&mut rt_memory);

    let mut constvec = ConstVec::<u32>::new_const(&mut alloc);
    constvec.push_const(&mut alloc, 1);
    constvec.push_const(&mut alloc, 2);

    let mut rtvec: Vec<u32> = vec![];

    rtvec.push(constvec.pop_const().unwrap());
    rtvec.push(constvec.pop_const().unwrap());

    assert_eq!(rtvec.len(), 2);
    rtvec.clear();
}

#[cfg(feature = "real_const_alloc")]
#[inline]
const fn into_actual_const_allocated() -> &'static [u32] {
    let mut memory = [0u8; 1024];
    let mut allocator = UnalignedConstStackAllocator::from_unique_slice(&mut memory);

    let mut constvec = ConstVec::<u32>::new_const(&mut allocator);

    constvec.push_const(&mut allocator, 1);
    constvec.push_const(&mut allocator, 2);
    constvec.push_const(&mut allocator, 3);
    constvec.push_const(&mut allocator, 4);

    constvec.into_const_allocated()
}

#[cfg(feature = "real_const_alloc")]
const fn something() -> &'static [u32] {
    let mut allocation = unsafe { intrinsics::const_allocate(8, 8) }.cast::<u64>();
    unsafe {
        allocation.write(0xaaaaaaaabbbbbbbb);
    }

    unsafe { core::slice::from_raw_parts(allocation.cast::<u32>(), 2) }
}
