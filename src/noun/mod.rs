//! A [noun] is a finite size binary tree with atoms as leaves.
//!
//! [noun]: https://urbit.org/docs/glossary/noun

pub mod types;

use crate::{atom::Atom, cell::Cell};
use std::{fmt::Debug, hash::Hash};

/// Interface to the noun data structure.
pub trait Noun<A, C>
where
    A: Atom,
    C: Cell<A, Self>,
    Self: Debug + Eq + Hash + Sized,
{
    fn get(&self, axis: usize) -> Option<&Self>;

    fn as_atom(&self) -> Result<&A, ()>;

    fn as_cell(&self) -> Result<&C, ()>;

    fn into_atom(self) -> Result<A, Self>;

    fn into_cell(self) -> Result<C, Self>;
}
