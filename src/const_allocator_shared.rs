use core::{
    intrinsics::{self, const_eval_select},
    ptr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocError;

#[macro_export(local_inner_macros)]
macro_rules! default_allocation_panic_str {
    () => {
        ::core::concat!(
            "Allocation or deallocation failed at user request.\n",
            "it may have been an out-of-memory situation, or a layout-size/layout-alignment discrepancy.\n",
            "Use the fallible allocation methods and handle the error manually instead.\n",
        )
    };
}

#[macro_export(local_inner_macros)]
macro_rules! const_alloc_panic {
    () => {{
        ::core::panic!($crate::default_allocation_panic_str!());
    }};

    ($($message: literal $(,)? )* $(,)?) => {{
        ::core::panic!(::core::concat!(
            $crate::default_allocation_panic_str!(),
            "\n",
            "Error-specific message:\n",
            $(::core::stringify!($message),)*
        ));
    }};
}

#[inline]
pub const fn cast_raw_slice<Src, Dst>(src: *const [Src]) -> *const [Dst] {
    assert!(
        size_of::<Src>() == size_of::<Dst>(),
        "this function can't be used with types of different size"
    );

    let data = src.cast::<Dst>();
    let len = src.len();

    ptr::slice_from_raw_parts(data, len)
}

#[inline]
pub const fn cast_raw_slice_mut<Src, Dst>(src: *mut [Src]) -> *mut [Dst] {
    const {
        assert!(
            size_of::<Src>() == size_of::<Dst>(),
            "this function can't be used with types of different size"
        );
    }

    let data = src.cast::<Dst>();
    let len = src.len();

    ptr::slice_from_raw_parts_mut(data, len)
}

/// Calculates which address after `base` is valid for a type of alignment `align` to exist.
#[inline]
#[must_use]
pub const fn next_aligned_addr(base: usize, align: usize) -> usize {
    // TODO !important: check for overflow and change signature to return Option<usize> on overflow.
    let total = base.next_multiple_of(align);

    // let's make sure our result is really aligned:
    debug_assert!(total % align == 0, "next_aligned_addr is not aligned");

    total
}

#[inline]
#[must_use]
pub const fn get_alignment_of_addr<T: ?Sized>(ptr: *const T) -> usize {
    #[inline]
    fn runtime(untyped_ptr: *const u8) -> usize {
        let trailing = untyped_ptr.addr().trailing_zeros();
        1usize << trailing
    }
    #[inline]
    const fn compiletime(untyped_ptr: *const u8) -> usize {
        let _ = untyped_ptr;
        1
    }
    let untyped_ptr = ptr.cast::<u8>();
    intrinsics::const_eval_select((untyped_ptr,), compiletime, runtime)
}
