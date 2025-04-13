use crate::align_twiddle::next_aligned_addr;
use crate::backing_alloc::BackingAllocation;
use alloc::alloc::handle_alloc_error;
use core::alloc::Layout;
use core::ptr::NonNull;

pub struct StackAllocator<'alloc> {
    buffer: BackingAllocation<'alloc>,
    pos: usize,
}

impl<'alloc> StackAllocator<'alloc> {
    pub const fn from_unique_slice(slice: &'alloc mut [u8]) -> Self {
        StackAllocator {
            buffer: BackingAllocation::from_unique_slice(slice),
            pos: 0,
        }
    }

    pub fn alloc_fallible(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let align = layout.align();
        let aligned_pos = next_aligned_addr(unsafe { self.buffer.as_ptr().add(self.pos) }.addr(), align) - self.buffer.as_ptr().addr();

        let new_pos = match aligned_pos.checked_add(layout.size()) {
            None => return Err(AllocError::OutOfMemory),
            Some(pos) => pos,
        };

        if new_pos > self.buffer.len() {
            return Err(AllocError::OutOfMemory);
        }

        let ptr = unsafe { self.buffer.as_mut_ptr().add(aligned_pos) };
        self.pos = new_pos;

        match NonNull::new(ptr) {
            None => Err(AllocError::OutOfMemory),
            Some(nn) => Ok(nn),
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> NonNull<u8> {
        match self.alloc_fallible(layout) {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(layout),
        }
    }

    pub fn alloc_zeroed_fallible(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let ptr = self.alloc_fallible(layout)?;

        unsafe {
            ptr.as_ptr().write_bytes(0, layout.size());
        }

        Ok(ptr)
    }

    pub fn alloc_zeroed(&mut self, layout: Layout) -> NonNull<u8> {
        match self.alloc_zeroed_fallible(layout) {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(layout),
        }
    }

    pub fn realloc_fallible(&mut self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<u8>, AllocError> {
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
        let new_ptr = self.alloc_fallible(new_layout)?;

        // Safety: both pointers are valid for their respective layouts
        unsafe {
            core::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), core::cmp::min(old_layout.size(), new_layout.size()));
        }

        Ok(new_ptr)
    }

    pub fn realloc(&mut self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> NonNull<u8> {
        match self.realloc_fallible(ptr, old_layout, new_layout) {
            Ok(ptr) => ptr,
            Err(_) => handle_alloc_error(new_layout),
        }
    }

    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        // check if this was the last allocation
        let ptr_addr = ptr.as_ptr() as usize;
        let buffer_start = self.buffer.as_ptr() as usize;
        let start_offset = ptr_addr - buffer_start;

        if start_offset + layout.size() == self.pos {
            self.pos = start_offset;
        }
        // we can't free individual allocations in the middle of the stack
    }

    pub fn into_inner(self) -> BackingAllocation<'alloc> {
        self.buffer
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
}
