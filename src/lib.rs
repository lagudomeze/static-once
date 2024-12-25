use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

/// A marker trait used to indicate that a type has been initialized.
/// The same trait cannot be implemented multiple times, so initialization can only occur once.
pub trait Inited {}

#[derive(Debug)]
pub struct StaticCell<T> {
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> StaticCell<T> {
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<T> StaticCell<T> {
    #[inline]
    pub unsafe fn assume_init_ref(&self) -> &T {
        unsafe { (&*self.value.get()).assume_init_ref() }
    }

    #[inline]
    pub unsafe fn assume_init_mut(&self) -> &mut T {
        unsafe { (&mut *self.value.get()).assume_init_mut() }
    }

    #[inline]
    pub unsafe fn write(&self, value: T) {
        unsafe {
            self.value.get().write(MaybeUninit::new(value));
        }
    }
}

impl<T> StaticCell<T>
where
    SharedRef<T>: Inited,
{
    pub fn get_ref(&self, _: &SharedRef<T>) -> &T {
        unsafe { self.assume_init_ref() }
    }

    pub fn get_mut_ref(&mut self) -> &mut T {
        unsafe { self.assume_init_mut() }
    }
}

unsafe impl<T> Sync for StaticCell<T> where T: Sync {}

#[derive(Debug, Copy)]
pub struct SharedRef<T> {
    _maker: PhantomData<T>,
}

unsafe impl<T> Sync for SharedRef<T> {

}

unsafe impl<T> Send for SharedRef<T> {

}

impl<T> Clone for SharedRef<T> {
    fn clone(&self) -> Self {
        Self {
            _maker: PhantomData,
        }
    }
}

impl<T> SharedRef<T> {
    /// Creates a new `SharedRef` instance. This function is unsafe because it
    /// assumes that the corresponding `StaticCell` has been properly initialized.
    pub unsafe fn unsafe_new() -> Self {
        Self {
            _maker: PhantomData,
        }
    }
}

#[macro_export]
macro_rules! exactly_init_once {
    ($ty:ty, $cell:ident, $value:expr) => {{
        use $crate::{Inited, SharedRef};

        // This trait is used to ensure that `Inited` is implemented only once.
        #[allow(non_local_definitions)]
        impl Inited for SharedRef<$ty> {

        }

        unsafe {
            $cell.write($value);
        }

        unsafe { SharedRef::<$ty>::unsafe_new() }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct A;

    struct B {
        a: SharedRef<A>,
    }

    #[test]
    fn it_works() {
        let cell_a = StaticCell::<A>::new();

        let a_ref = exactly_init_once!(A, cell_a, A);

        let cell_b = StaticCell::<B>::new();

        let a = a_ref.clone();
        let b_ref = exactly_init_once!(B, cell_b, B {
            a,
        });

        let b = cell_b.get_ref(&b_ref);

        assert_eq!(cell_a.get_ref(&b.a), cell_a.get_ref(&a_ref));
    }
}
