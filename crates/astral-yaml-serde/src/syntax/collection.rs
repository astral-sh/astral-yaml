use std::marker::PhantomData;

/// The state of a selected collection.
#[derive(Debug)]
pub struct Collection<Kind> {
    pub(crate) len: usize,
    pub(crate) kind: PhantomData<Kind>,
}

/// Mapping cursor state.
#[derive(Debug)]
pub enum Mapping {}

/// Sequence cursor state.
#[derive(Debug)]
pub enum Sequence {}
