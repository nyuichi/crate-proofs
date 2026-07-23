use core::mem::MaybeUninit;
use creusot_std::prelude::{ensures, logic, trusted, DeepModel, Int};

pub trait VecStorage<T> {
    #[logic]
    fn capacity(self) -> Int;

    fn borrow(&self) -> &[MaybeUninit<T>];

    fn borrow_mut(&mut self) -> &mut [MaybeUninit<T>];

    #[ensures(result@ == self.capacity())]
    fn runtime_capacity(&self) -> usize;
}

pub struct VecStorageInner<T> {
    pub(crate) buffer: T,
}

pub(crate) type OwnedVecStorage<T, const N: usize> = VecStorageInner<[MaybeUninit<T>; N]>;

impl<T, const N: usize> VecStorage<T> for OwnedVecStorage<T, N> {
    #[logic(open)]
    fn capacity(self) -> Int {
        N.deep_model()
    }

    #[trusted]
    fn borrow(&self) -> &[MaybeUninit<T>] {
        &self.buffer
    }

    #[trusted]
    fn borrow_mut(&mut self) -> &mut [MaybeUninit<T>] {
        &mut self.buffer
    }

    #[ensures(result@ == self.capacity())]
    fn runtime_capacity(&self) -> usize {
        N
    }
}
