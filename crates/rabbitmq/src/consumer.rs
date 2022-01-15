use std::marker::PhantomData;

#[derive(Debug)]
pub struct Consumer<T, Q> {
    _p: PhantomData<(T, Q)>,
}
