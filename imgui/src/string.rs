use std::borrow::{Borrow, Cow};
use std::ffi::CStr;
use std::ops::{Deref, Index, RangeFull};
use std::os::raw::c_char;
use std::str;
use std::{fmt, ptr};
// use std::ptr::NonNull;
// use std::alloc::{GlobalAlloc, Layout, System};

/// this is the unsafe cell upon which we build our abstraction.
#[repr(C)]
#[derive(Debug)]
pub struct UiBuffer {
    /* 
    pub buf: NonNull<u8>,
    pub buf_len: usize,
    pub buf_cap: usize,
    */
    pub buffer: Vec<u8>,
    pub max_len: usize,
}

impl UiBuffer {
    /// Creates a new max buffer with the given length.
    pub const fn new(max_len: usize) -> Self {
        Self {
            buffer: Vec::new(),
            /* 
            buf: NonNull::dangling(),
            buf_len: 0,
            buf_cap: 0, 
            */
            max_len,
        }
    }

    /// Internal method to push a single text to our scratch buffer.
    pub fn scratch_txt(&mut self, txt: impl AsRef<str>) -> *const core::ffi::c_char {
        self.refresh_buffer();

        let start_of_substr = self.push(txt);
        unsafe { self.offset(start_of_substr) }
    }

    /// Internal method to push an option text to our scratch buffer.
    pub fn scratch_txt_opt(&mut self, txt: Option<impl AsRef<str>>) -> *const core::ffi::c_char {
        match txt {
            Some(v) => self.scratch_txt(v),
            None => ptr::null(),
        }
    }

    /// Helper method, same as [`Self::scratch_txt`] but for two strings
    pub fn scratch_txt_two(
        &mut self,
        txt_0: impl AsRef<str>,
        txt_1: impl AsRef<str>,
    ) -> (*const core::ffi::c_char, *const core::ffi::c_char) {
        self.refresh_buffer();

        let first_offset = self.push(txt_0);
        let second_offset = self.push(txt_1);

        unsafe { (self.offset(first_offset), self.offset(second_offset)) }
    }

    /// Helper method, same as [`Self::scratch_txt`] but with one optional value
    pub fn scratch_txt_with_opt(
        &mut self,
        txt_0: impl AsRef<str>,
        txt_1: Option<impl AsRef<str>>,
    ) -> (*const core::ffi::c_char, *const core::ffi::c_char) {
        match txt_1 {
            Some(value) => self.scratch_txt_two(txt_0, value),
            None => (self.scratch_txt(txt_0), ptr::null()),
        }
    }

    /// Attempts to clear the buffer if it's over the maximum length allowed.
    /// This is to prevent us from making a giant vec over time.
    pub fn refresh_buffer(&mut self) {
        if self.buffer.len() > self.max_len {
            self.buffer.clear();
        }
        /* 
        if self.buf_len > self.max_len {
            // let buf_ptr= self.buf.as_ptr();
            self.buf_len = 0;
        }
        */
    }

    /// Given a position, gives an offset from the start of the scatch buffer.
    ///
    /// # Safety
    /// This can return a pointer to undefined data if given a `pos >= self.buffer.len()`.
    /// This is marked as unsafe to reflect that.
    pub unsafe fn offset(&self, pos: usize) -> *const core::ffi::c_char {
        self.buffer.as_ptr().add(pos) as *const _
        // self.buf.as_ptr().add(pos) as *const _
    }

    /// Pushes a new scratch sheet text and return the byte index where the sub-string
    /// starts.
    pub fn push(&mut self, txt: impl AsRef<str>) -> usize {
        let len = self.buffer.len();
        self.buffer.extend(txt.as_ref().as_bytes());
        self.buffer.push(b'\0');

        len
        /* 
        // println!("old buf: 0x{:x}, len: {}, cap: {}", self.buf.as_ptr() as usize, self.buf_len, self.buf_cap);
        let len = self.buf_len;
        let tgt_len = self.buf_len + txt.as_ref().len() + 1;
        if tgt_len > self.buf_cap {
            let dealloc_old = self.buf_cap > 0;
            let new_cap = match self.buf_cap {
                0 => 1 << (usize::BITS - tgt_len.leading_zeros() + 1),
                v => v * 2
            };

            // make new allocation
            let layout = unsafe { Layout::from_size_align_unchecked(self.buf_cap, align_of::<usize>()) };
            let new_layout = unsafe { Layout::from_size_align_unchecked(new_cap, align_of::<usize>()) };
            let new_ptr = unsafe { System.alloc(new_layout) };
            unsafe { 
                std::ptr::copy_nonoverlapping(self.buf.as_ptr(), new_ptr, self.buf_len);
            }
            if dealloc_old {
                unsafe { System.dealloc(self.buf.as_ptr(), layout) }
            }
            self.buf = unsafe { NonNull::new_unchecked(new_ptr) };
            self.buf_cap = new_cap;
        }
        // insert text + null terminator
        unsafe { 
            std::ptr::copy_nonoverlapping(
                txt.as_ref().as_ptr(),
                self.buf.as_ptr().add(self.buf_len), 
                txt.as_ref().len()
            );
            *self.buf.as_ptr().add(self.buf_len + txt.as_ref().len()) = 0;
        }
        self.buf_len += txt.as_ref().len() + 1;
        // println!("old buf: 0x{:x}, len: {}, cap: {}", self.buf.as_ptr() as usize, self.buf_len, self.buf_cap);
       len 
        */
    }
}

/// A UTF-8 encoded, growable, implicitly nul-terminated string.
#[derive(Clone, Hash, Ord, Eq, PartialOrd, PartialEq)]
pub struct ImString(pub(crate) Vec<u8>);

impl ImString {
    /// Creates a new `ImString` from an existing string.
    pub fn new<T: Into<String>>(value: T) -> ImString {
        unsafe {
            let mut s = ImString::from_utf8_unchecked(value.into().into_bytes());
            s.refresh_len();
            s
        }
    }

    /// Creates a new empty `ImString` with a particular capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> ImString {
        let mut v = Vec::with_capacity(capacity + 1);
        v.push(b'\0');
        ImString(v)
    }

    /// Converts a vector of bytes to a `ImString` without checking that the string contains valid
    /// UTF-8
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the vector contains valid UTF-8 and no null terminator.
    #[inline]
    pub unsafe fn from_utf8_unchecked(mut v: Vec<u8>) -> ImString {
        v.push(b'\0');
        ImString(v)
    }

    /// Converts a vector of bytes to a `ImString` without checking that the string contains valid
    /// UTF-8
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the vector contains valid UTF-8 and a null terminator.
    #[inline]
    pub unsafe fn from_utf8_with_nul_unchecked(v: Vec<u8>) -> ImString {
        ImString(v)
    }

    /// Truncates this `ImString`, removing all contents
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
        self.0.push(b'\0');
    }

    /// Appends the given character to the end of this `ImString`
    #[inline]
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        self.push_str(ch.encode_utf8(&mut buf));
    }

    /// Appends a given string slice to the end of this `ImString`
    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.0.pop();
        self.0.extend(string.bytes());
        self.0.push(b'\0');
        unsafe {
            self.refresh_len();
        }
    }

    /// Returns the capacity of this `ImString` in bytes
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity() - 1
    }

    /// Returns the capacity of this `ImString` in bytes, including the implicit null byte
    #[inline]
    pub fn capacity_with_nul(&self) -> usize {
        self.0.capacity()
    }

    /// Ensures that the capacity of this `ImString` is at least `additional` bytes larger than the
    /// current length.
    ///
    /// The capacity may be increased by more than `additional` bytes.
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Ensures that the capacity of this `ImString` is at least `additional` bytes larger than the
    /// current length
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    /// Returns a raw pointer to the underlying buffer
    #[inline]
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr() as *const c_char
    }

    /// Returns a raw mutable pointer to the underlying buffer.
    ///
    /// If the underlying data is modified, `refresh_len` *must* be called afterwards.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr() as *mut c_char
    }

    /// Updates the underlying buffer length based on the current contents.
    ///
    /// This function *must* be called if the underlying data is modified via a pointer
    /// obtained by `as_mut_ptr`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the this ImString contains valid UTF-8 and a null
    /// terminator.
    #[inline]
    pub unsafe fn refresh_len(&mut self) {
        let len = CStr::from_ptr(self.0.as_ptr() as *const c_char)
            .to_bytes_with_nul()
            .len();
        self.0.set_len(len);
    }
}

impl Default for ImString {
    #[inline]
    fn default() -> ImString {
        ImString(vec![b'\0'])
    }
}

impl From<String> for ImString {
    #[inline]
    fn from(s: String) -> ImString {
        ImString::new(s)
    }
}

impl<'a> From<ImString> for Cow<'a, ImStr> {
    #[inline]
    fn from(s: ImString) -> Cow<'a, ImStr> {
        Cow::Owned(s)
    }
}

impl<'a> From<&'a ImString> for Cow<'a, ImStr> {
    #[inline]
    fn from(s: &'a ImString) -> Cow<'a, ImStr> {
        Cow::Borrowed(s)
    }
}

impl<'a, T: ?Sized + AsRef<ImStr>> From<&'a T> for ImString {
    #[inline]
    fn from(s: &'a T) -> ImString {
        s.as_ref().to_owned()
    }
}

impl AsRef<ImStr> for ImString {
    #[inline]
    fn as_ref(&self) -> &ImStr {
        self
    }
}

impl Borrow<ImStr> for ImString {
    #[inline]
    fn borrow(&self) -> &ImStr {
        self
    }
}

impl AsRef<str> for ImString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl Borrow<str> for ImString {
    #[inline]
    fn borrow(&self) -> &str {
        self.to_str()
    }
}

impl Index<RangeFull> for ImString {
    type Output = ImStr;
    #[inline]
    fn index(&self, _index: RangeFull) -> &ImStr {
        self
    }
}

impl fmt::Debug for ImString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.to_str(), f)
    }
}

impl fmt::Display for ImString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.to_str(), f)
    }
}

impl Deref for ImString {
    type Target = ImStr;
    #[inline]
    fn deref(&self) -> &ImStr {
        // as_ptr() is used, because we need to look at the bytes to figure out the length
        // self.0.len() is incorrect, because there might be more than one nul byte in the end, or
        // some interior nuls in the data
        unsafe {
            &*(CStr::from_ptr(self.0.as_ptr() as *const c_char) as *const CStr as *const ImStr)
        }
    }
}

impl fmt::Write for ImString {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c);
        Ok(())
    }
}

/// A UTF-8 encoded, implicitly nul-terminated string slice.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ImStr([u8]);

impl<'a> Default for &'a ImStr {
    #[inline]
    fn default() -> &'a ImStr {
        static SLICE: &[u8] = &[0];
        unsafe { ImStr::from_utf8_with_nul_unchecked(SLICE) }
    }
}

impl fmt::Debug for ImStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for ImStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.to_str(), f)
    }
}

impl ImStr {
    /// Wraps a raw UTF-8 encoded C string
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the pointer is not null and it points to a
    /// null-terminated UTF-8 string valid for the duration of the arbitrary lifetime 'a.
    #[inline]
    pub unsafe fn from_ptr_unchecked<'a>(ptr: *const c_char) -> &'a ImStr {
        ImStr::from_cstr_unchecked(CStr::from_ptr(ptr))
    }

    /// Converts a slice of bytes to an imgui-rs string slice without checking for valid UTF-8 or
    /// null termination.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the slice contains valid UTF-8 and a null terminator.
    #[inline]
    pub unsafe fn from_utf8_with_nul_unchecked(bytes: &[u8]) -> &ImStr {
        &*(bytes as *const [u8] as *const ImStr)
    }

    /// Converts a CStr reference to an imgui-rs string slice without checking for valid UTF-8.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the CStr reference contains valid UTF-8.
    #[inline]
    pub unsafe fn from_cstr_unchecked(value: &CStr) -> &ImStr {
        &*(value.to_bytes_with_nul() as *const [u8] as *const ImStr)
    }

    /// Converts an imgui-rs string slice to a raw pointer
    #[inline]
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr() as *const c_char
    }

    /// Converts an imgui-rs string slice to a normal string slice
    #[inline]
    pub fn to_str(&self) -> &str {
        self.sanity_check();
        unsafe { str::from_utf8_unchecked(&self.0[..(self.0.len() - 1)]) }
    }

    /// Returns true if the imgui-rs string slice is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.0.len() != 0);
        self.0.len() == 1
    }

    // TODO: if this is too slow, avoid the UTF8 validation except if we'd
    // already be doing O(n) stuff.
    #[inline]
    fn sanity_check(&self) {
        debug_assert!(
            str::from_utf8(&self.0).is_ok()
                && !self.0.is_empty()
                && !self.0[..(self.0.len() - 1)].contains(&0u8)
                && self.0[self.0.len() - 1] == 0,
            "bad ImStr: {:?}",
            &self.0
        );
    }
}

impl AsRef<CStr> for ImStr {
    #[inline]
    fn as_ref(&self) -> &CStr {
        // Safety: our safety requirements are a superset of CStr's, so this is fine
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.0) }
    }
}

impl AsRef<ImStr> for ImStr {
    #[inline]
    fn as_ref(&self) -> &ImStr {
        self
    }
}

impl AsRef<str> for ImStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl<'a> From<&'a ImStr> for Cow<'a, ImStr> {
    #[inline]
    fn from(s: &'a ImStr) -> Cow<'a, ImStr> {
        Cow::Borrowed(s)
    }
}

impl ToOwned for ImStr {
    type Owned = ImString;
    #[inline]
    fn to_owned(&self) -> ImString {
        self.sanity_check();
        ImString(self.0.to_owned())
    }
}

#[test]
fn test_imstring_constructors() {
    let s = ImString::new("test");
    assert_eq!(s.0, b"test\0");

    let s = ImString::with_capacity(100);
    assert_eq!(s.0, b"\0");

    let s = unsafe { ImString::from_utf8_unchecked(vec![b't', b'e', b's', b't']) };
    assert_eq!(s.0, b"test\0");

    let s = unsafe { ImString::from_utf8_with_nul_unchecked(vec![b't', b'e', b's', b't', b'\0']) };
    assert_eq!(s.0, b"test\0");
}

#[test]
fn test_imstring_operations() {
    let mut s = ImString::new("test");
    s.clear();
    assert_eq!(s.0, b"\0");
    s.push('z');
    assert_eq!(s.0, b"z\0");
    s.push('ä');
    assert_eq!(s.0, b"z\xc3\xa4\0");
    s.clear();
    s.push_str("imgui-rs");
    assert_eq!(s.0, b"imgui-rs\0");
    s.push_str("öä");
    assert_eq!(s.0, b"imgui-rs\xc3\xb6\xc3\xa4\0");
}

#[test]
fn test_imstring_fmt_write() {
    use std::fmt::Write;
    let mut s = ImString::default();
    let _ = write!(s, "format {:02x}", 0x42);
    assert_eq!(s.0, b"format 42\0");
}

#[test]
fn test_imstring_refresh_len() {
    let mut s = ImString::new("testing");
    unsafe {
        let mut ptr = s.as_mut_ptr() as *mut u8;
        ptr = ptr.wrapping_add(2);
        *ptr = b'z';
        ptr = ptr.wrapping_add(1);
        *ptr = b'\0';
    }
    assert_eq!(s.0, b"tez\0ing\0");
    unsafe { s.refresh_len() };
    assert_eq!(s.0, b"tez\0");
}

#[test]
fn test_imstring_interior_nul() {
    let s = ImString::new("test\0ohno");
    assert_eq!(s.0, b"test\0");
    assert_eq!(s.to_str(), "test");
    assert!(!s.is_empty());

    let s = ImString::new("\0ohno");
    assert_eq!(s.to_str(), "");
    assert!(s.is_empty());
}
