use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct StaticCell<T> {
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for StaticCell<T> where T: Sync {}

impl<T> StaticCell<T> {
    pub const fn new() -> Self
    where
        Self: 'static,
    {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    #[inline]
    pub unsafe fn get(&'static self) -> &'static T {
        (&*self.value.get()).assume_init_ref()
    }

    #[inline]
    pub unsafe fn set(&'static self, value: T) -> &'static mut T {
        (&mut *self.value.get()).write(value)
    }
}

pub trait StaticInit {
    type Item: 'static;

    const HOLDER: &'static StaticCell<Self::Item>;

    ///
    ///
    ///
    /// # Safety can be called only once
    ///
    /// # Arguments
    ///
    /// * `value`: value for init
    ///
    /// returns: Inited<Self>
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
