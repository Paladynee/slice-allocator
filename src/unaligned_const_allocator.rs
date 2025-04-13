use crate::const_alloc_panic;
use crate::const_allocator_shared::AllocError;
use crate::unaligned_generic_buffer::UnalignedGenericBuffer;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr;
use core::ptr::NonNull;

pub struct UnalignedConstStackAllocator<'buffer> {
    buffer: UnalignedGenericBuffer<'buffer, u8>,
    pos: usize,
}

impl<'alloc> UnalignedConstStackAllocator<'alloc> {
    #[inline]
    #[must_use]
    pub const fn from_unique_slice(slice: &'alloc mut [u8]) -> Self {
        UnalignedConstStackAllocator {
            buffer: UnalignedGenericBuffer::from_unique_slice(slice),
            pos: 0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_unique_uninit_slice(slice: &'alloc mut [MaybeUninit<u8>]) -> Self {
        UnalignedConstStackAllocator {
            buffer: UnalignedGenericBuffer::from_unique_uninit_slice(slice),
            pos: 0,
        }
    }

    /// # Errors
    ///
    /// - If the requested layout is too large to fit in the underlying allocation
    #[inline]
    pub const fn alloc_const_unaligned_fallible(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        // pointers don't have alignment, nor an integer value in const.
        let _ = layout.align();
        let len = layout.size();

        if self.pos + len > self.buffer.unaligned_len() {
            return Err(AllocError);
        }

        let ptr = unsafe { self.buffer.as_unaligned_mut_ptr().add(self.pos) };
        self.pos += len;

        match NonNull::new(ptr) {
            None => Err(AllocError),
            Some(nn) => Ok(nn),
        }
    }

    #[inline]
    pub const fn alloc_const_unaligned(&mut self, layout: Layout) -> NonNull<u8> {
        match self.alloc_const_unaligned_fallible(layout) {
            Ok(ptr) => ptr,
            Err(_) => const_alloc_panic!("alloc_const_unaligned_fallible failed"),
        }
    }

    /// #  Errors
    ///
    /// - If the requested layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// The pointer must point to a valid memory location that was allocated by this allocator.
    #[inline]
    pub const unsafe fn realloc_const_unaligned_fallible(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<u8>, AllocError> {
        let _ = old_layout.align();
        let _ = new_layout.align();

        // use *const T::offset_from to calculate the difference, as pointers aren't available as usize
        // in compile-time, and any conversion from a pointer to any integer is immediate comptime UB.
        let offset = unsafe { ptr.as_ptr().offset_from_unsigned(self.buffer.as_unaligned_ptr()) };

        if offset + old_layout.size() == self.pos {
            // if new size is smaller, we can just update the position
            if new_layout.size() <= old_layout.size() {
                self.pos = offset + new_layout.size();
                return Ok(ptr);
            }

            // otherwise check if we have enough space
            let additional_space = new_layout.size() - old_layout.size();
            if self.pos + additional_space <= self.buffer.unaligned_len() {
                self.pos += additional_space;
                return Ok(ptr);
            }
        }

        // if we can't resize in place, allocate a new block and copy
        let new_ptr = match self.alloc_const_unaligned_fallible(new_layout) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        // copy the old data to the new location

        let old_size = old_layout.size();
        let new_size = new_layout.size();
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), if old_size < new_size { old_size } else { new_size });
        }

        Ok(new_ptr)
    }

    /// #  Panics
    ///
    /// - If the requested layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// The pointer must point to a valid memory location that was allocated by this allocator.
    #[inline]
    pub const unsafe fn realloc_const_unaligned(&mut self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> NonNull<u8> {
        match unsafe { self.realloc_const_unaligned_fallible(ptr, old_layout, new_layout) } {
            Ok(p) => p,
            Err(_) => const_alloc_panic!("realloc_const_unaligned failed"),
        }
    }

    /// # Safety
    ///
    /// The pointer must point to a valid memory location that was allocated by this allocator.
    /// The layout must be the same as the one used to allocate the memory.
    #[inline]
    pub const unsafe fn dealloc_const_unaligned(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let _ = layout.align();
        let offset = unsafe { ptr.as_ptr().offset_from_unsigned(self.buffer.as_unaligned_ptr()) };

        // lets rewrite the contents of pointer to 0xAA
        if cfg!(debug_assertions) {
            unsafe {
                ptr::write_bytes(ptr.as_ptr(), 0xAA, layout.size());
            }
        }

        // if the pointer is at the end of the buffer, we can just update the position
        if offset + layout.size() == self.pos {
            self.pos = offset;
        }

        // otherwise we can't deallocate in the middle of the buffer
    }
}
