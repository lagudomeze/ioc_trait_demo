use std::cell::UnsafeCell;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct StaticPlace<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> StaticPlace<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::new(value)),
        }
    }

    pub const fn uninit() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub const fn as_mut_ptr(&self) -> *mut MaybeUninit<T> {
        self.inner.get()
    }
}

unsafe impl<T: Sync> Sync for StaticPlace<T> {}

unsafe impl<T: Send> Send for StaticPlace<T> {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_static_place() {}
}
