use core::alloc::AllocError;
use core::alloc::Layout;
use core::intrinsics;
use core::ptr;
use core::ptr::NonNull;

pub struct ConstAllocator;

impl ConstAllocator {
    /// # Panics
    ///
    /// Panics if called in runtime.
    ///
    /// # Safety
    ///
    /// Layout's alignment must be a power of two.
    pub const unsafe fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = layout.size();
        let p = unsafe { intrinsics::const_allocate(size, layout.align()) };

        let slice = core::ptr::slice_from_raw_parts_mut(p, size);
        let nn = NonNull::new(slice).expect("intrinsics::const_allocate returned null, ConstAllocator can't be used in runtime.");

        Ok(nn)
    }

    /// # Safety
    ///
    /// Layout's alignment must be a power of two.
    pub const unsafe fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let Ok(p) = (unsafe { self.allocate(layout) }) else {
            return Err(AllocError);
        };

        unsafe {
            ptr::write_bytes(p.as_ptr().cast::<u8>(), 0, layout.size());
        }

        Ok(p)
    }

    /// # Safety
    ///
    /// Given pointer and layout must match an allocation made by this allocator and
    /// hasn't been invalidated yet.
    pub const unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            intrinsics::const_deallocate(ptr.as_ptr(), layout.size(), layout.align());
        }
    }

    /// # Safety
    ///
    /// Given pointer and old layout must match an allocation made by this allocator and
    /// hasn't been invalidated yet.
    pub const unsafe fn reallocate(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let Ok(new_ptr) = (unsafe { self.allocate(new_layout) }) else {
            return Err(AllocError);
        };

        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr().cast::<u8>(), old_layout.size());
            self.deallocate(ptr, old_layout);
        }

        Ok(new_ptr)
    }
}
