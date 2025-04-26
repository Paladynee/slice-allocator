use crate::backing_alloc::BackingAllocation;
use crate::unaligned_generic_buffer::UnalignedGenericBuffer;
use core::alloc::Layout;
use core::mem::{MaybeUninit, size_of};
use core::ptr;
use core::ptr::NonNull;
#[derive(Debug, Clone, Copy)]
pub struct ExperimentalAllocError(pub &'static str);
const PORT_BYTE_ALIGNMENT: usize = 8;
const PORT_BYTE_ALIGNMENT_MASK: usize = PORT_BYTE_ALIGNMENT - 1;
#[repr(C)]
struct BlockLink {
    next_free: Option<NonNull<BlockLink>>,
    block_size: TaggedUsize,
}
impl BlockLink {
    const HEAP_STRUCT_SIZE: usize = (size_of::<Self>() + PORT_BYTE_ALIGNMENT_MASK) & !PORT_BYTE_ALIGNMENT_MASK;
    const MINIMUM_BLOCK_SIZE: usize = Self::HEAP_STRUCT_SIZE * 2;
}
#[derive(Debug, Clone, Copy)]
struct TaggedUsize(usize);
impl TaggedUsize {
    const ALLOCATED_BIT: usize = 1 << (usize::BITS as usize - 1);
    #[inline]
    const fn new(size: usize, allocated: bool) -> Self {
        let mut val = size & !Self::ALLOCATED_BIT;
        if allocated {
            val |= Self::ALLOCATED_BIT;
        }
        Self(val)
    }
    #[inline]
    const fn size(self) -> usize {
        self.0 & !Self::ALLOCATED_BIT
    }
    #[inline]
    const fn is_allocated(self) -> bool {
        (self.0 & Self::ALLOCATED_BIT) != 0
    }
    #[inline]
    const fn set_allocated(&mut self) {
        self.0 |= Self::ALLOCATED_BIT;
    }
    #[inline]
    const fn set_free(&mut self) {
        self.0 &= !Self::ALLOCATED_BIT;
    }
    #[inline]
    const fn is_valid(self) -> bool {
        (self.0 & Self::ALLOCATED_BIT) == 0
    }
}
const fn block_size_is_valid(x: usize) -> bool {
    TaggedUsize(x).is_valid()
}
const fn block_is_allocated(block: &BlockLink) -> bool {
    block.block_size.is_allocated()
}
const fn allocate_block(block: &mut BlockLink) {
    block.block_size.set_allocated();
}
const fn free_block(block: &mut BlockLink) {
    block.block_size.set_free();
}
trait OptionExt<T> {
    unsafe fn unwrap_debug_checked(self) -> T;
}
impl<T> OptionExt<T> for Option<T> {
    #[inline]
    #[track_caller]
    unsafe fn unwrap_debug_checked(self) -> T {
        match self {
            None => {
                #[cfg(debug_assertions)]
                panic!("Option::unwrap called on `None` value at {:?}", core::panic::Location::caller());
                #[cfg(not(debug_assertions))]
                unsafe {
                    core::hint::unreachable_unchecked()
                }
            }
            Some(val) => val,
        }
    }
}
trait NonNullExt<T: ?Sized> {
    unsafe fn new_debug_checked(ptr: *mut T) -> NonNull<T>;
}
impl<T: ?Sized> NonNullExt<T> for NonNull<T> {
    unsafe fn new_debug_checked(ptr: *mut T) -> Self {
        unsafe { Self::new(ptr).unwrap_debug_checked() }
    }
}
pub struct ExperimentalAllocator<'buf> {
    mem: UnalignedGenericBuffer<'buf, u8>,
    free_head: Option<NonNull<BlockLink>>,
    end_marker: Option<NonNull<BlockLink>>,
}
impl<'buf> ExperimentalAllocator<'buf> {
    #[inline]
    #[must_use]
    pub fn from_unique_slice(mem: &'buf mut [u8]) -> Self {
        let ugb = UnalignedGenericBuffer::from_unique_slice(mem);
        ExperimentalAllocator::from_raw_parts(ugb)
    }
    #[inline]
    #[must_use]
    pub fn from_unique_uninit_slice(mem: &'buf mut [MaybeUninit<u8>]) -> Self {
        let ugb = UnalignedGenericBuffer::from_unique_uninit_slice(mem);
        ExperimentalAllocator::from_raw_parts(ugb)
    }
    #[inline]
    #[must_use]
    pub fn from_backing_allocation(mem: BackingAllocation<'buf>) -> Self {
        let ugb = UnalignedGenericBuffer::from_backing_allocation(mem);
        ExperimentalAllocator::from_raw_parts(ugb)
    }
    #[inline]
    fn init_heap(&mut self) {
        let buf_ptr = self.mem.as_unaligned_ptr();
        let buf_len = self.mem.unaligned_len();
        let heap_start = buf_ptr.map_addr(|addr| Self::align_up(addr, PORT_BYTE_ALIGNMENT));
        let heap_end = unsafe { buf_ptr.add(buf_len) };
        let heap_end_addr = heap_end.addr();
        let end_marker_addr = {
            let candidate = heap_end_addr.saturating_sub(BlockLink::HEAP_STRUCT_SIZE);
            Self::align_up(candidate, PORT_BYTE_ALIGNMENT)
        };
        if end_marker_addr < heap_start.addr() || (end_marker_addr + BlockLink::HEAP_STRUCT_SIZE) > heap_end_addr {
            self.free_head = None;
            self.end_marker = None;
            return;
        }
        let dummy_head = heap_start as *mut BlockLink;
        let first_block_addr = heap_start.map_addr(|addr| Self::align_up(addr + BlockLink::HEAP_STRUCT_SIZE, PORT_BYTE_ALIGNMENT));
        let end_marker = end_marker_addr as *mut BlockLink;
        unsafe {
            (*end_marker).block_size = TaggedUsize::new(0, false);
            (*end_marker).next_free = None;
        }
        let first_block = first_block_addr as *mut BlockLink;
        let first_block_size = end_marker_addr.saturating_sub(first_block_addr.addr());
        unsafe {
            (*first_block).block_size = TaggedUsize::new(first_block_size, false);
            (*first_block).next_free = Some(NonNull::new_debug_checked(end_marker));
        }
        unsafe {
            (*dummy_head).block_size = TaggedUsize::new(0, false);
            (*dummy_head).next_free = Some(NonNull::new_debug_checked(first_block));
        }
        self.free_head = unsafe { Some(NonNull::new_debug_checked(dummy_head)) };
        self.end_marker = unsafe { Some(NonNull::new_debug_checked(end_marker)) };
    }
    #[inline]
    const fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }
    #[inline]
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, ExperimentalAllocError> {
        let wanted_size = layout.size();
        let align = layout.align().max(PORT_BYTE_ALIGNMENT);
        if wanted_size == 0 {
            return Err(ExperimentalAllocError("requested allocation size is zero"));
        }
        if self.free_head.is_none() {
            return Err(ExperimentalAllocError("free list is not initialized or buffer too small"));
        }
        let mut total_size = wanted_size.saturating_add(BlockLink::HEAP_STRUCT_SIZE);
        if (total_size & (align - 1)) != 0 {
            total_size += align - (total_size & (align - 1));
        }
        if !block_size_is_valid(total_size) {
            return Err(ExperimentalAllocError("allocation size overflowed usize or MSB set"));
        }
        let mut prev = unsafe { self.free_head.unwrap_debug_checked() };
        let mut curr = unsafe { prev.as_ref().next_free.unwrap_debug_checked() };
        while {
            let curr_ref = unsafe { curr.as_ref() };
            (curr_ref.block_size.size()) < total_size && curr_ref.next_free.is_some()
        } {
            prev = curr;
            curr = unsafe { curr.as_ref().next_free.unwrap_debug_checked() };
        }
        if curr == unsafe { self.end_marker.unwrap_debug_checked() } {
            return Err(ExperimentalAllocError("no suitable free block found"));
        }
        let curr_block_size = unsafe { curr.as_ref().block_size.size() };
        if curr_block_size - total_size > BlockLink::MINIMUM_BLOCK_SIZE {
            let new_block_addr = (curr.as_ptr() as usize + total_size) as *mut BlockLink;
            unsafe {
                (*new_block_addr).block_size = TaggedUsize::new(curr_block_size - total_size, false);
                (*new_block_addr).next_free = curr.as_ref().next_free;
                (*curr.as_ptr()).block_size = TaggedUsize::new(total_size, false);
                (*curr.as_ptr()).next_free = None;
                allocate_block(&mut *curr.as_ptr());
            }
        } else {
            unsafe {
                (*prev.as_ptr()).next_free = curr.as_ref().next_free;
                (*curr.as_ptr()).next_free = None;
                allocate_block(&mut *curr.as_ptr());
            }
        }
        let user_ptr = unsafe { curr.as_ptr().cast::<u8>().add(BlockLink::HEAP_STRUCT_SIZE) };
        let slice = ptr::slice_from_raw_parts_mut(user_ptr, wanted_size);
        Ok(unsafe { NonNull::new_debug_checked(slice) })
    }
    #[inline]
    pub unsafe fn free(&mut self, ptr: NonNull<u8>, _layout: Layout) {
        let block_ptr = unsafe { ptr.as_ptr().sub(BlockLink::HEAP_STRUCT_SIZE).cast::<BlockLink>() };
        debug_assert!(block_is_allocated(unsafe { &*block_ptr }), "Double free or corruption detected");
        let next_free_ptr = unsafe { &raw mut (*block_ptr).next_free };
        unsafe { next_free_ptr.write_unaligned(None) };
        free_block(unsafe { &mut *block_ptr });
        self.insert_block_into_free_list(block_ptr);
    }
    #[inline]
    fn insert_block_into_free_list(&mut self, block: *mut BlockLink) {
        let mut prev = unsafe { self.free_head.unwrap_debug_checked() };
        let mut curr = unsafe { prev.as_ref().next_free.unwrap_debug_checked() };
        while curr < unsafe { NonNull::new_debug_checked(block) } && curr != unsafe { self.end_marker.unwrap_debug_checked() } {
            prev = curr;
            curr = unsafe { curr.as_ref().next_free.unwrap_debug_checked() };
        }
        let prev_end = unsafe { (prev.as_ptr() as usize) + (prev.as_ref().block_size.size()) };
        let mut block = block;
        if prev_end == block as usize {
            unsafe {
                (*prev.as_ptr()).block_size = TaggedUsize::new(prev.as_ref().block_size.size() + (*block).block_size.size(), false);
                block = prev.as_ptr();
            }
        } else {
            unsafe {
                (*prev.as_ptr()).next_free = Some(NonNull::new_debug_checked(block));
            }
        }
        let block_end = unsafe { (block as usize) + ((*block).block_size.size()) };
        if block_end == curr.as_ptr() as usize {
            unsafe {
                (*block).block_size = TaggedUsize::new((*block).block_size.size() + curr.as_ref().block_size.size(), false);
                (*block).next_free = curr.as_ref().next_free;
            }
        } else {
            unsafe {
                (*block).next_free = Some(curr);
            }
        }
    }
    #[inline]
    fn from_raw_parts(ugb: UnalignedGenericBuffer<'buf, u8>) -> Self {
        let mut alloc = ExperimentalAllocator {
            mem: ugb,
            free_head: None,
            end_marker: None,
        };
        alloc.init_heap();
        alloc
    }
}
mod tests {
    use super::*;
    use {alloc::boxed::Box, alloc::vec};
    fn create_allocator(buf_size: usize) -> ExperimentalAllocator<'static> {
        let mem_box = vec![0u8; buf_size].into_boxed_slice();
        let leaked: &'static mut [u8] = Box::leak(mem_box);
        ExperimentalAllocator::from_unique_slice(leaked)
    }
    #[test]
    fn test_alloc_and_free() {
        let mut allocator = create_allocator(1024);
        let alloc_ptr = allocator.alloc(Layout::from_size_align(100, 1).unwrap());
        assert!(alloc_ptr.is_ok(), "Allocation should succeed: {:?}", alloc_ptr.err());
        let slice_ptr = alloc_ptr.unwrap();
        let ptr = slice_ptr.cast::<u8>();
        unsafe {
            core::ptr::write_bytes(ptr.as_ptr(), 0xAA, 100);
        }
        unsafe {
            allocator.free(ptr, Layout::from_size_align(100, 1).unwrap());
        }
        let alloc_ptr2 = allocator.alloc(Layout::from_size_align(100, 1).unwrap());
        assert!(alloc_ptr2.is_ok(), "Allocation after free should succeed: {:?}", alloc_ptr2.err());
    }
    #[test]
    fn test_allocation_too_big() {
        let mut allocator = create_allocator(64);
        let alloc_ptr = allocator.alloc(Layout::from_size_align(1024, 1).unwrap());
        assert!(alloc_ptr.is_err(), "Allocation should fail for too big request");
        if let Err(ExperimentalAllocError(msg)) = alloc_ptr {
            assert!(msg == "no suitable free block found" || msg == "free list is not initialized or buffer too small");
        } else {
            panic!("Expected ExperimentalAllocError");
        }
    }
    #[test]
    fn test_allocates_multiple_blocks() {
        let mut allocator = create_allocator(1024);
        let block1 = allocator.alloc(Layout::from_size_align(100, 1).unwrap());
        let block2 = allocator.alloc(Layout::from_size_align(200, 1).unwrap());
        let block3 = allocator.alloc(Layout::from_size_align(300, 1).unwrap());
        assert!(block1.is_ok(), "Block1 allocation should succeed");
        assert!(block2.is_ok(), "Block2 allocation should succeed");
        assert!(block3.is_ok(), "Block3 allocation should succeed");
        unsafe {
            let ptr2 = block2.unwrap().cast::<u8>();
            allocator.free(ptr2, Layout::from_size_align(200, 1).unwrap());
        }
        let block4 = allocator.alloc(Layout::from_size_align(150, 1).unwrap());
        assert!(block4.is_ok(), "Block4 allocation should succeed using freed space");
    }
}