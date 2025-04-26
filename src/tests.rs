use crate::backing_alloc::BackingAllocation;
use crate::const_vec::ConstVec;
use crate::unaligned_const_allocator::UnalignedConstStackAllocator;
use alloc::vec;
use alloc::vec::Vec;
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

#[cfg(feature = "real_const_alloc")]
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
fn rust_intrinsic_const_allocate_can_escape_to_runtime_safely_test() {
    let data = const { rust_intrinsic_const_allocate_can_escape_to_runtime_safely() };
    // endianness is unknown
    // assert!(data[0] == 0xbbbbbbbb);
    // assert!(data[1] == 0xaaaaaaaa);
    assert!(data.len() == 2);
    assert!(data[0] != 0);
    assert!(data[1] != 0);
}

#[cfg(feature = "allocator_api")]
#[test]
fn slice_allocator_runtime_test() {
    let val = slice_allocator_runtime();
    assert!(val == 15);
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
#[inline]
const fn rust_intrinsic_const_allocate_can_escape_to_runtime_safely() -> &'static [u32] {
    let mut allocation = unsafe { intrinsics::const_allocate(8, 8) }.cast::<u64>();
    unsafe {
        allocation.write(0xaaaaaaaabbbbbbbb);
    }

    unsafe { core::slice::from_raw_parts(allocation.cast::<u32>(), 2) }
}

#[cfg(feature = "allocator_api")]
#[inline]
pub fn slice_allocator_runtime() -> u64 {
    use crate::slice_allocator::SingleThreadedSliceAllocator;
    extern crate std;

    let mut rt_memory = vec![0; 1024];
    let mut alloc = unsafe { SingleThreadedSliceAllocator::from_unique_slice(&mut rt_memory) };
    let mut subvec: Vec<u8, &SingleThreadedSliceAllocator> = Vec::with_capacity_in(32, &alloc);

    for i in 0..32 {
        subvec.push(i as u8);
    }

    let sum = subvec.iter().fold(0, |acc, &x| acc + x as u64);
    subvec.clear();
    drop(subvec);

    let mut subvec2: Vec<u16, &SingleThreadedSliceAllocator> = Vec::with_capacity_in(16, &alloc);
    let mut subvec3: Vec<u32, &SingleThreadedSliceAllocator> = Vec::with_capacity_in(16, &alloc);
    for i in 0..16 {
        subvec2.push(i as u16);
        subvec3.push(i as u32);
    }

    let sum2 = subvec2.iter().fold(0, |acc, &x| acc + x as u64);
    let sum3 = subvec3.iter().fold(0, |acc, &x| acc + x as u64);

    drop(subvec2);
    drop(subvec3);

    drop(alloc);

    std::println!("backing memory: {:?}", rt_memory);

    if sum + sum2 + sum3 != 0 { 15 } else { 0 }
}
