use crate::backing_alloc::BackingAllocation;
use crate::unaligned_generic_buffer::UnalignedGenericBuffer;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr;
use core::ptr::NonNull;
use core::slice;

#[cfg(feature = "allocator_api")]
use core::alloc::AllocError as StdAllocError;
#[cfg(feature = "allocator_api")]
use core::alloc::Allocator;

pub struct StackAllocator<'buf> {
    mem: UnalignedGenericBuffer<'buf, u8>,
    pos: usize,
}

pub struct SingleThreadedSliceAllocator<'buf> {
    alloc: UnsafeCell<StackAllocator<'buf>>,
}

impl<'buf> SingleThreadedSliceAllocator<'buf> {
    /// # Safety
    ///
    /// Must not be used in multithreaded contexts
    #[inline]
    pub const unsafe fn from_unique_slice(slice: &'buf mut [u8]) -> Self {
        let mem = UnalignedGenericBuffer::from_unique_slice(slice);
        SingleThreadedSliceAllocator::from_raw_parts(mem, 0)
    }

    /// # Safety
    ///
    /// Must not be used in multithreaded contexts
    #[inline]
    pub const unsafe fn from_unique_uninit_slice(slice: &'buf mut [MaybeUninit<u8>]) -> Self {
        let mem = UnalignedGenericBuffer::from_unique_uninit_slice(slice);
        SingleThreadedSliceAllocator::from_raw_parts(mem, 0)
    }

    /// # Safety
    ///
    /// Must not be used in multithreaded contexts
    #[inline]
    #[must_use]
    pub const unsafe fn from_backing_allocation(backing_alloc: BackingAllocation<'buf>) -> Self {
        let mem = UnalignedGenericBuffer::from_backing_allocation(backing_alloc);
        SingleThreadedSliceAllocator::from_raw_parts(mem, 0)
    }

    /// # Safety
    ///
    /// Must not be used in multithreaded contexts
    #[inline]
    #[must_use]
    pub const unsafe fn from_unaligned_generic_buffer(mem: UnalignedGenericBuffer<'buf, u8>) -> Self {
        SingleThreadedSliceAllocator::from_raw_parts(mem, 0)
    }

    const fn from_raw_parts(mem: UnalignedGenericBuffer<'buf, u8>, pos: usize) -> Self {
        let alloc = StackAllocator { mem, pos };
        let usc = UnsafeCell::new(alloc);
        SingleThreadedSliceAllocator { alloc: usc }
    }
}

#[cfg(feature = "allocator_api")]
unsafe impl Allocator for SingleThreadedSliceAllocator<'_> {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, StdAllocError> {
        let allocator: &mut StackAllocator = unsafe { &mut *self.alloc.get() };
        let align = layout.align();
        let size = layout.size();

        let aligned_end = allocator
            .mem
            .as_next_aligned_mut_ptr_for(allocator.pos, align)
            .map_addr(|addr| addr + size);

        // check if the next valid address is in the buffer
        let end_past_one = unsafe { allocator.mem.as_unaligned_ptr().add(allocator.mem.valid_len()) };

        if aligned_end.cast_const() > end_past_one {
            return Err(StdAllocError);
        }

        let start = aligned_end.map_addr(|addr| addr - size);
        let slice = unsafe { slice::from_raw_parts_mut(start, size) };

        let Some(nn) = NonNull::new(slice) else { return Err(StdAllocError) };

        allocator.pos += size;
        Ok(nn)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, StdAllocError> {
        let ptr = self.allocate(layout)?;
        unsafe {
            let dest = ptr.cast::<u8>().as_ptr();
            ptr::write_bytes(dest, 0, layout.size());
        }
        Ok(ptr)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let allocator: &mut StackAllocator = unsafe { &mut *self.alloc.get() };

        let offset = unsafe { ptr.as_ptr().offset_from_unsigned(allocator.mem.as_unaligned_ptr()) };
        let size = layout.size();

        if cfg!(debug_assertions) {
            // rewrite the contents of the buffer to 0xAA for better debugging experience.
            let debug_ptr = unsafe { allocator.mem.as_unaligned_mut_ptr().add(offset) };

            // ⚠️ debug_ptr may be unaligned!
            // ⚠️ UB: ptr::write_bytes(debug_ptr, 0xAA, size);

            let mut i = 0;
            while i < size {
                let byte_ptr = unsafe { debug_ptr.add(i) };
                unsafe { byte_ptr.write_unaligned(0xAA) };
                i += 1;
            }
        }

        // if the pointer is at the end of the buffer, we can just update the position
        if offset + size == allocator.pos {
            allocator.pos = offset;
        }

        // otherwise, we can't really deallocate from the middle of the buffer
    }
}
