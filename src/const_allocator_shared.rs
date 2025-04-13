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
