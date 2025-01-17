use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct StaticCell<T> {
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for StaticCell<T> where T: Sync {}

impl<T> Default for StaticCell<T> where T: 'static {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StaticCell<T> {
    pub const fn new() -> Self
    where
        Self: 'static,
    {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// # Safety
    /// This function is unsafe because it's up to the caller to ensure that
    /// 1. the value is initialized.
    /// 2. no other caller *write* it (call set method) at the same time.
    #[inline]
    pub unsafe fn get(&'static self) -> &'static T {
        (*self.value.get()).assume_init_ref()
    }

    /// # Safety
    /// This function is unsafe because it's up to the caller to ensure that
    /// 1. no other caller *read* or *write* it (call get/set method) at the same time.
    /// 2. set method should be called only once. (maybe call set multiple times is ok, but it's not recommended)
    #[inline]
    pub unsafe fn set(&'static self, value: T) {
        (*self.value.get()).write(value);
    }
}

pub trait StaticInit {
    type Item: 'static;

    #[allow(clippy::declare_interior_mutable_const)]
    const HOLDER: &'static StaticCell<Self::Item>;

    /// # Safety
    /// can be called only once
    ///
    /// # Arguments
    ///
    /// * `value`: value for init
    ///
    /// returns: Inited<Self> which can be used to get the reference of static value safely (and it is zero cost).
    ///
    /// # Examples
    ///
    /// ```
    ///  use static_once::{StaticCell, StaticInit};
    ///  struct A;
    ///
    ///  static __A__: StaticCell<A> = StaticCell::new();
    ///
    ///  impl StaticInit for A {
    ///     type Item = Self;
    ///     const HOLDER: &'static StaticCell<Self::Item> = &__A__;
    ///  }
    ///
    ///  let inited = unsafe { A::init(A) };
    ///
    ///  // here inited.get() is zero cost to get the reference of static value
    ///  // you can clone/copy the inited everywhere.
    ///  println!("{:p}", inited.get());
    /// ```
    #[allow(clippy::borrow_interior_mutable_const)]
    unsafe fn init(value: Self::Item) -> Inited<Self>
    where
        Self: Sized,
    {
        Self::HOLDER.set(value);
        Inited { _marker: PhantomData }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Inited<B> {
    _marker: PhantomData<B>,
}

impl<B> Inited<B> {
    /// Safety: when you get an `Inited`, you are guaranteed that `init` has been called.
    #[inline]
    #[allow(clippy::borrow_interior_mutable_const)]
    pub fn get(&self) -> &'static B::Item
    where
        B: StaticInit,
    {
        unsafe { B::HOLDER.get() }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct A;

    static __A__: StaticCell<A> = StaticCell::new();

    impl StaticInit for A {
        type Item = Self;
        const HOLDER: &'static StaticCell<Self::Item> = &__A__;
    }

    #[test]
    fn it_works() {
        let t = Box::new(StaticCell::<usize>::new());
        let t = Box::leak(t);

        let a = unsafe { t.get() };

        println!("{:p}", a);

        let inited = unsafe { A::init(A) };
        println!("{:p}", inited.get());
        println!("{:p}", A::HOLDER);
    }
}
