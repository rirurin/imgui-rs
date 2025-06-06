//! Internal raw utilities (don't use unless you know what you're doing!)

use std::{
    ffi::c_int,
    ops::{ Index, IndexMut },
    slice
};

/// A generic version of the raw imgui-sys ImVector struct types
#[repr(C)]
pub struct ImVector<T> {
    size: c_int,
    capacity: c_int,
    pub(crate) data: *mut T,
}

impl<T> ImVector<T> {
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.size as usize) }
    }

    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data, self.size as usize) }
    }

    pub fn replace_from_slice(&mut self, data: &[T]) {
        unsafe {
            sys::igMemFree(self.data as *mut _);

            let buffer_ptr = sys::igMemAlloc(std::mem::size_of_val(data)) as *mut T;
            buffer_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

            self.size = data.len() as i32;
            self.capacity = data.len() as i32;
            self.data = buffer_ptr;
        }
    }

    pub fn len(&self) -> usize { self.size as usize }
    pub fn capacity(&self) -> usize { self.capacity as usize }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(unsafe { &*self.data.add(index) })
        } else {
            None
        }
        
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            Some(unsafe { &mut *self.data.add(index) })
        } else {
            None
        }
    }

}

impl<T> Index<usize> for ImVector<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if index < self.len() {
            unsafe { &*self.data.add(index) }
        } else {
            panic!("{} is out of bounds for ImVector of length {}", index, self.len())
        }
    }
}

impl<T> IndexMut<usize> for ImVector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.len() {
            unsafe { &mut *self.data.add(index) }
        } else {
            panic!("{} is out of bounds for ImVector of length {}", index, self.len())
        }
    }
}

#[test]
#[cfg(test)]
fn test_imvector_memory_layout() {
    use std::mem;
    assert_eq!(
        mem::size_of::<ImVector<u8>>(),
        mem::size_of::<sys::ImVector_char>()
    );
    assert_eq!(
        mem::align_of::<ImVector<u8>>(),
        mem::align_of::<sys::ImVector_char>()
    );
    use sys::ImVector_char;
    type VectorChar = ImVector<u8>;
    macro_rules! assert_field_offset {
        ($l:ident, $r:ident) => {
            assert_eq!(
                memoffset::offset_of!(VectorChar, $l),
                memoffset::offset_of!(ImVector_char, $r)
            );
        };
    }
    assert_field_offset!(size, Size);
    assert_field_offset!(capacity, Capacity);
    assert_field_offset!(data, Data);
}

/// Marks a type as a transparent wrapper over a raw type
pub trait RawWrapper {
    /// Wrapped raw type
    type Raw;
    /// Returns an immutable reference to the wrapped raw value
    ///
    /// # Safety
    ///
    /// It is up to the caller to use the returned raw reference without causing undefined
    /// behaviour or breaking safety rules.
    unsafe fn raw(&self) -> &Self::Raw;
    /// Returns a mutable reference to the wrapped raw value
    ///
    /// # Safety
    ///
    /// It is up to the caller to use the returned mutable raw reference without causing undefined
    /// behaviour or breaking safety rules.
    unsafe fn raw_mut(&mut self) -> &mut Self::Raw;
}

/// Casting from/to a raw type that has the same layout and alignment as the target type
///
/// # Safety
///
/// Each function outlines its own safety contract, which generally is
/// that the cast from `T` to `Self` is valid.
pub unsafe trait RawCast<T>: Sized {
    /// Casts an immutable reference from the raw type
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the cast is valid.
    #[inline]
    unsafe fn from_raw(raw: &T) -> &Self {
        &*(raw as *const _ as *const Self)
    }
    /// Casts a mutable reference from the raw type
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the cast is valid.
    #[inline]
    unsafe fn from_raw_mut(raw: &mut T) -> &mut Self {
        &mut *(raw as *mut _ as *mut Self)
    }
    /// Casts an immutable reference to the raw type
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the cast is valid.
    #[inline]
    unsafe fn raw(&self) -> &T {
        &*(self as *const _ as *const T)
    }
    /// Casts a mutable reference to the raw type
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the cast is valid.
    #[inline]
    unsafe fn raw_mut(&mut self) -> &mut T {
        &mut *(self as *mut _ as *mut T)
    }
}

/// A primary data type
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataType {
    I8 = sys::ImGuiDataType_S8,
    U8 = sys::ImGuiDataType_U8,
    I16 = sys::ImGuiDataType_S16,
    U16 = sys::ImGuiDataType_U16,
    I32 = sys::ImGuiDataType_S32,
    U32 = sys::ImGuiDataType_U32,
    I64 = sys::ImGuiDataType_S64,
    U64 = sys::ImGuiDataType_U64,
    F32 = sys::ImGuiDataType_Float,
    F64 = sys::ImGuiDataType_Double,
}

/// Primitive type marker.
///
/// If this trait is implemented for a type, it is assumed to have *exactly* the same
/// representation in memory as the primitive value described by the associated `KIND` constant.
///
/// # Safety
/// The `DataType` *must* have the same representation as the primitive value of `KIND`.
pub unsafe trait DataTypeKind: Copy {
    const KIND: DataType;
}
unsafe impl DataTypeKind for i8 {
    const KIND: DataType = DataType::I8;
}
unsafe impl DataTypeKind for u8 {
    const KIND: DataType = DataType::U8;
}
unsafe impl DataTypeKind for i16 {
    const KIND: DataType = DataType::I16;
}
unsafe impl DataTypeKind for u16 {
    const KIND: DataType = DataType::U16;
}
unsafe impl DataTypeKind for i32 {
    const KIND: DataType = DataType::I32;
}
unsafe impl DataTypeKind for u32 {
    const KIND: DataType = DataType::U32;
}
unsafe impl DataTypeKind for i64 {
    const KIND: DataType = DataType::I64;
}
unsafe impl DataTypeKind for u64 {
    const KIND: DataType = DataType::U64;
}
unsafe impl DataTypeKind for f32 {
    const KIND: DataType = DataType::F32;
}
unsafe impl DataTypeKind for f64 {
    const KIND: DataType = DataType::F64;
}

unsafe impl DataTypeKind for usize {
    #[cfg(target_pointer_width = "16")]
    const KIND: DataType = DataType::U16;

    #[cfg(target_pointer_width = "32")]
    const KIND: DataType = DataType::U32;

    #[cfg(target_pointer_width = "64")]
    const KIND: DataType = DataType::U64;

    // Fallback for when we are on a weird system width
    //
    #[cfg(not(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    )))]
    compile_error!("cannot impl DataTypeKind for usize: unsupported target pointer width. supported values are 16, 32, 64");
}

unsafe impl DataTypeKind for isize {
    #[cfg(target_pointer_width = "16")]
    const KIND: DataType = DataType::I16;

    #[cfg(target_pointer_width = "32")]
    const KIND: DataType = DataType::I32;

    #[cfg(target_pointer_width = "64")]
    const KIND: DataType = DataType::I64;

    // Fallback for when we are on a weird system width
    //
    #[cfg(not(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    )))]
    compile_error!("cannot impl DataTypeKind for isize: unsupported target pointer width. supported values are 16, 32, 64");
}
