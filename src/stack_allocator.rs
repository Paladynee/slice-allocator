use crate::align_twiddle::next_aligned_addr;
use crate::backing_alloc::BackingAllocation;
use crate::const_allocator_shared::AllocError;
use alloc::alloc::handle_alloc_error;
use core::alloc::Layout;
use core::cmp;
use core::mem::MaybeUninit;
use core::ptr;
use core::ptr::NonNull;

pub struct StackAllocator<'alloc> {
    buffer: BackingAllocation<'alloc>,
    pos: usize,
}

impl<'alloc> StackAllocator<'alloc> {
    #[inline]
    pub const fn from_unique_slice(slice: &'alloc mut [u8]) -> Self {
        StackAllocator {
            buffer: BackingAllocation::from_unique_slice(slice),
            pos: 0,
        }
    }

    #[inline]
    pub const fn from_unique_uninit_slice(slice: &'alloc mut [MaybeUninit<u8>]) -> Self {
        StackAllocator {
            buffer: BackingAllocation::from_unique_uninit_slice(slice),
            pos: 0,
        }
    }

    /// # Errors
    ///
    /// - If the requested layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// Layout must have a valid alignment that won't overflow the next
    /// valid aligned address calculation.
    #[inline]
    pub unsafe fn alloc_fallible(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let align = layout.align();

        let buffer_start_addr = self.buffer.as_ptr().addr();
        let current_addr = buffer_start_addr + self.pos;

        let aligned_addr = next_aligned_addr(current_addr, align);
        let aligned_pos = aligned_addr - buffer_start_addr;

        let new_pos = match aligned_pos.checked_add(layout.size()) {
            None => return Err(AllocError),
            Some(pos) => pos,
        };

        if new_pos > self.buffer.len() {
            return Err(AllocError);
        }

        let ptr = unsafe { self.buffer.as_mut_ptr().add(aligned_pos) };

        // safety: BackingAllocation is based on a valid slice
        let nn = unsafe { NonNull::new_unchecked(ptr) };

        self.pos = new_pos;
        Ok(nn)
    }

    /// # Panics
    ///
    /// - If the requested layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// Layout must have a valid alignment that won't overflow the next
    /// valid aligned address calculation.
    #[inline]
    pub unsafe fn alloc(&mut self, layout: Layout) -> NonNull<u8> {
        match unsafe { self.alloc_fallible(layout) } {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(layout),
        }
    }

    /// # Errors
    ///
    /// - If the requested layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// Layout must have a valid alignment that won't overflow the next
    /// valid aligned address calculation.
    #[inline]
    pub unsafe fn alloc_zeroed_fallible(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let ptr = unsafe { self.alloc_fallible(layout) }?;

        unsafe {
            ptr.as_ptr().write_bytes(0, layout.size());
        }

        Ok(ptr)
    }

    /// # Panics
    ///
    /// - If the requested layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// Layout must have a valid alignment that won't overflow the next
    /// valid aligned address calculation.
    #[inline]
    pub unsafe fn alloc_zeroed(&mut self, layout: Layout) -> NonNull<u8> {
        match unsafe { self.alloc_zeroed_fallible(layout) } {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(layout),
        }
    }

    /// # Errors
    ///
    /// - If the new layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// The new layout must have a valid alignment that won't overflow the next
    /// valid aligned address calculation.
    #[inline]
    pub unsafe fn realloc_fallible(&mut self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<u8>, AllocError> {
        // check if this was the last allocation
        let old_ptr_addr = ptr.as_ptr() as usize;
        let buffer_start = self.buffer.as_ptr() as usize;
        let old_start_offset = old_ptr_addr - buffer_start;

        if old_start_offset + old_layout.size() == self.pos {
            // if new size is smaller, we can just update the position
            if new_layout.size() <= old_layout.size() {
                self.pos = old_start_offset + new_layout.size();
                return Ok(ptr);
            }

            // otherwise check if we have enough space
            let additional_space = new_layout.size() - old_layout.size();
            if self.pos + additional_space <= self.buffer.len() {
                self.pos += additional_space;
                return Ok(ptr);
            }
        }

        // if we can't resize in place, allocate a new block and copy
        let new_ptr = unsafe { self.alloc_fallible(new_layout) }?;

        // Safety: both pointers are valid for their respective layouts
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), cmp::min(old_layout.size(), new_layout.size()));
        }

        Ok(new_ptr)
    }

    /// # Panics
    ///
    /// - If the new layout is too large to fit in the underlying allocation
    ///
    /// # Safety
    ///
    /// The new layout must have a valid alignment that won't overflow the next
    /// valid aligned address calculation.
    #[inline]
    pub unsafe fn realloc(&mut self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> NonNull<u8> {
        match unsafe { self.realloc_fallible(ptr, old_layout, new_layout) } {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(new_layout),
        }
    }

    /// # Safety
    ///
    /// The pointer must be derived from this allocator.
    #[inline]
    pub const unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        // use *const T::offset_from to calculate the difference, as pointers aren't available as usize
        // in compile-time, and any conversion from a pointer to any integer is immediate comptime UB.
        let offset = unsafe { ptr.as_ptr().offset_from_unsigned(self.buffer.as_ptr()) };

        // check if the deallocated memory is at the top of the stack
        if offset + layout.size() == self.pos {
            // we can reclaim this memory
            self.pos = offset;
        }

        // we can't free individual allocations in the middle of the stack
    }

    #[inline]
    #[must_use]
    pub const fn into_inner(self) -> BackingAllocation<'alloc> {
        self.buffer
    }
}
