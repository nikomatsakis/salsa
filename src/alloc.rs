use std::ptr::NonNull;

/// A box but without the uniqueness guarantees.
pub struct Alloc<T: ?Sized> {
    data: NonNull<T>,
}

impl<T: ?Sized> Alloc<T> {
    pub fn new(data: T) -> Self
    where
        T: Sized,
    {
        let data = Box::new(data);
        let data = Box::into_raw(data);
        Alloc {
            data: unsafe { NonNull::new_unchecked(data) },
        }
    }

    pub fn as_raw(&self) -> NonNull<T> {
        self.data
    }

    pub unsafe fn as_ref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }
}

impl<T: ?Sized> From<Box<T>> for Alloc<T> {
    fn from(data: Box<T>) -> Self {
        let data = Box::into_raw(data);
        Alloc {
            data: unsafe { NonNull::new_unchecked(data) },
        }
    }
}

impl<T: ?Sized> Drop for Alloc<T> {
    fn drop(&mut self) {
        let data: *mut T = self.data.as_ptr();
        let data: Box<T> = unsafe { Box::from_raw(data) };
        drop(data);
    }
}

unsafe impl<T: ?Sized> Send for Alloc<T> where T: Send {}

unsafe impl<T: ?Sized> Sync for Alloc<T> where T: Sync {}
