use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use crate::{backing_alloc::BackingAllocation, const_allocator_shared::get_alignment_of_addr, unaligned_generic_buffer::UnalignedGenericBuffer};

#[derive(Debug)]
pub struct AllocatorNode {
    next: Option<NonNull<AllocatorNode>>,
    prev: Option<NonNull<AllocatorNode>>,
}

pub struct WeirdAllocator<'buf> {
    mem: UnalignedGenericBuffer<'buf, u8>,
}

impl<'buf> WeirdAllocator<'buf> {
    #[inline]
    #[must_use]
    pub const fn from_unique_slice(mem: &'buf mut [u8]) -> Self {
        let ugb = UnalignedGenericBuffer::from_unique_slice(mem);
        Self::from_raw_parts(ugb)
    }

    #[inline]
    #[must_use]
    pub const fn from_backing_allocation(mem: BackingAllocation<'buf>) -> Self {
        let ugb = UnalignedGenericBuffer::from_backing_allocation(mem);
        Self::from_raw_parts(ugb)
    }

    #[inline]
    #[must_use]
    pub const fn from_unique_uninit_slice(mem: &'buf mut [MaybeUninit<u8>]) -> Self {
        let ugb = UnalignedGenericBuffer::from_unique_uninit_slice(mem);
        Self::from_raw_parts(ugb)
    }

    #[inline]
    const fn from_raw_parts(mem: UnalignedGenericBuffer<'buf, u8>) -> Self {
        // this tells us the alignment of the base address of our unaligned buffer
        let align = get_alignment_of_addr(mem.as_unaligned_ptr());

        Self { mem }
    }
}
