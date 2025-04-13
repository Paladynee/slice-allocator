use core::marker::PhantomData;
use core::slice;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BaseAddress<T: ?Sized> {
    base: *mut T,
    len: usize,
}

impl<T: ?Sized> BaseAddress<T> {
    #[inline]
    pub const fn from_ptr_len(base: *mut T, len: usize) -> Self {
        // TODO: unnecessary structure name repetition false positive (or others are false negatives?)
        Self { base, len }
    }

    #[inline]
    #[must_use]
    pub const fn base_const(&self) -> *const T {
        self.base
    }

    #[inline]
    pub const fn base_mut(&mut self) -> *mut T {
        self.base
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait ByAddrConst<T: ?Sized> {
    /// # Safety
    ///
    /// The given base address must be derived from an allocated object,
    /// and the self offset must be encapsulated inside an allocated object,
    /// aka. it may not wrap around the end of the address space.
    unsafe fn by_addr(&self, base: BaseAddress<u8>) -> *const T;
}

pub trait ByAddrMut<T: ?Sized> {
    /// # Safety
    ///
    /// The given base address must be derived from an allocated object,
    /// and the self offset must be encapsulated inside an allocated object,
    /// aka. it may not wrap around the end of the address space.
    unsafe fn by_addr_mut(&mut self, base: BaseAddress<u8>) -> *mut T;
}

macro_rules! make_primitive_aliases {
    // base case
    () => {};

    // no mut before ty
    (struct $name: ident: $ty: ty {}, $($rest:tt)*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $name<T> {
            inner: $ty,
            _marker: PhantomData<T>,
        }

        impl<T> ByAddrConst<T> for $name<T> {
            #[inline]
            unsafe fn by_addr(&self, base: BaseAddress<u8>) -> *const T {
                debug_assert!(
                    self.inner as usize <= base.len(),
                    "Offset {} is larger than base length {}",
                    self.inner,
                    base.len()
                );
                unsafe { base.base_const().byte_add(self.inner as usize) }.cast::<T>()
            }
        }

        make_primitive_aliases!($($rest)*);
    };

    // mut before ty
    (struct $name: ident: mut $ty: ty {}, $($rest:tt)*) => {
        // call the non-mut version first
        make_primitive_aliases!(struct $name: $ty {},);

        impl<T> ByAddrMut<T> for $name<T> {
            #[inline]
            unsafe fn by_addr_mut(&mut self, mut base: BaseAddress<u8>) -> *mut T {
                debug_assert!(
                    self.inner as usize <= base.len(),
                    "Offset {} is larger than base length {}",
                    self.inner,
                    base.len()
                );
                unsafe { base.base_mut().byte_add(self.inner as usize) }.cast::<T>()
            }
        }

        make_primitive_aliases!($($rest)*);
    };
}

make_primitive_aliases!(
    struct U8AddrConst: u8 {},
    struct U8AddrMut: mut u8 {},
    struct U16AddrConst: u16 {},
    struct U16AddrMut: mut u16 {},
    struct U32AddrConst: u32 {},
    struct U32AddrMut: mut u32 {},
    struct U64AddrConst: u64 {},
    struct U64AddrMut: mut u64 {},
    struct U128AddrConst: u128 {},
    struct U128AddrMut: mut u128 {},
);

macro_rules! make_custom_slices {
    // base case
    () => {};

    // const slice
    (struct $slice_name: ident: $addr_ty: ident, $len_ty: ty {}, $($rest:tt)*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $slice_name<T> {
            offset: $addr_ty<T>,
            len: $len_ty,
        }

        impl<T> $slice_name<T> {
            #[inline]
            #[must_use]
            pub const fn len(&self) -> $len_ty {
                self.len
            }

            #[inline]
            #[must_use]
            pub const fn is_empty(&self) -> bool {
                self.len() == 0
            }
        }

        impl<T> ByAddrConst<[T]> for $slice_name<T> {
            #[inline]
            unsafe fn by_addr(&self, base: BaseAddress<u8>) -> *const [T] {
                let data = unsafe { self.offset.by_addr(base) };
                let len = self.len() as usize;
                unsafe { slice::from_raw_parts(data, len) }
            }
        }

        make_custom_slices!($($rest)*);
    };

    // mutable slice
    (struct $slice_name: ident: mut $addr_ty: ident, $len_ty: ty {}, $($rest:tt)*) => {
        // call the non-mut version first
        make_custom_slices!(struct $slice_name: $addr_ty, $len_ty {},);

        impl<T> ByAddrMut<[T]> for $slice_name<T> {
            #[inline]
            unsafe fn by_addr_mut(&mut self, base: BaseAddress<u8>) -> *mut [T] {
                let data = unsafe { self.offset.by_addr_mut(base) };
                let len = self.len() as usize;
                unsafe { slice::from_raw_parts_mut(data, len) }
            }
        }

        make_custom_slices!($($rest)*);
    };
}

make_custom_slices!(
    struct U8SliceConst: U8AddrConst, u8 {},
    struct U8SliceMut: mut U8AddrMut, u8 {},
    struct U16SliceConst: U16AddrConst, u16 {},
    struct U16SliceMut: mut U16AddrMut, u16 {},
    struct U32SliceConst: U32AddrConst, u32 {},
    struct U32SliceMut: mut U32AddrMut, u32 {},
    struct U64SliceConst: U64AddrConst, u64 {},
    struct U64SliceMut: mut U64AddrMut, u64 {},
    struct U128SliceConst: U128AddrConst, u128 {},
    struct U128SliceMut: mut U128AddrMut, u128 {},
);
