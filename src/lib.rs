#![no_std]
#![cfg_attr(feature = "nightly_unstable_const_heap", feature(const_heap))]
#![cfg_attr(feature = "core_intrinsics", feature(core_intrinsics))]

// used for slice_allocator
extern crate alloc;

pub mod align_twiddle;
pub mod aligned_generic_buffer;
pub mod aligned_raw_slice;
pub mod backing_alloc;
pub mod const_allocator_shared;
pub mod const_vec;
pub mod unaligned_const_allocator;
pub mod unaligned_generic_buffer;

#[cfg(test)]
mod tests;

// // disable #![no_std] to run
// #[test]
// pub fn test1() {
//     use aligned_generic_buffer::AlignedGenericBuffer;
//     use core::alloc::Layout;
//     use slice_allocator::StackAllocator;
//     use unaligned_generic_buffer::UnalignedGenericBuffer;

//     let something = || -> Option<(u32, u32)> {
//         let mut buffer = [0u8; 13];
//         println!("buffer len: {}", buffer.len());
//         let mut alloc = StackAllocator::from_unique_slice(&mut buffer);

//         let ptr = match unsafe { alloc.alloc_zeroed_fallible(Layout::new::<u32>()) } {
//             Ok(s) => s,
//             Err(_) => return None,
//         };

//         println!("ptr: {:?}", ptr);

//         let cast = ptr.cast::<u32>();

//         let prev = unsafe { cast.read() };
//         unsafe {
//             cast.as_ptr().write(42);
//         }
//         let next = unsafe { cast.read() };

//         let mut agb: AlignedGenericBuffer<'_, u64> = AlignedGenericBuffer::from_aligned_mut_raw_slice(
//             UnalignedGenericBuffer::from_backing_allocation(alloc.into_inner()).as_mut_aligned_raw_slice(),
//         );

//         let ptr = agb.as_mut_ptr();
//         let len = agb.len();
//         let mut i = 0;
//         while i < len {
//             unsafe {
//                 let ptr2 = ptr.add(i);
//                 ptr2.write(0xaaaaaaaaaaaaaaaa);
//             }
//             i += 1;
//         }

//         println!("underlying bytes: {:?}", buffer.as_slice());

//         Some((prev, next))
//     };
//     assert_eq!(something().unwrap(), (0, 42));
// }
