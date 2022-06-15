use crate::{
    Atom as _Atom, Cell as _Cell, Cue as _Cue, IntoNoun as _IntoNoun, Jam as _Jam, Mug as _Mug,
    Noun as _Noun,
};
use std::hash::{Hash, Hasher};

pub enum Noun {
    Atom(Atom),
    Cell(Cell),
}

impl _Cue for Noun {
    type Error = ();

    fn cue(_jammed_val: Vec<u8>) -> Result<Self, <Self as _Cue>::Error> {
        todo!()
    }
}

impl Eq for Noun {}

impl Hash for Noun {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        todo!()
    }
}

impl _Jam for Noun {
    type Error = ();

    fn jam(self) -> Result<Vec<u8>, <Self as _Jam>::Error> {
        todo!()
    }
}

impl _Mug for Noun {}

impl _Noun for Noun {
    type Atom = Atom;
    type Cell = Cell;
    type Error = ();

    fn into_atom(self) -> Result<<Self as _Noun>::Atom, <Self as _Noun>::Error> {
        match self {
            Self::Atom(atom) => Ok(atom),
            _ => Err(()),
        }
    }

    fn into_cell(self) -> Result<<Self as _Noun>::Cell, <Self as _Noun>::Error> {
        match self {
            Self::Cell(cell) => Ok(cell),
            _ => Err(()),
        }
    }
}

impl PartialEq for Noun {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

pub struct Atom(Vec<u8>);

impl _Atom for Atom {
    type Error = ();

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn as_u64(&self) -> Result<u64, <Self as _Atom>::Error> {
        todo!()
    }
}

impl _IntoNoun for Atom {
    type Error = ();
    type Noun = Noun;

    fn into_noun(self) -> Result<Self::Noun, Self::Error> {
        Ok(Noun::Atom(self))
    }
}

pub struct Cell {
    head: Option<NounPtr>,
    tail: Option<NounPtr>,
}

impl _Cell for Cell {
    type Noun = NounPtr;

    fn get(&self, _idx: usize) -> Option<<Self as _Cell>::Noun> {
        todo!()
    }

    fn into_parts(self) -> (Option<<Self as _Cell>::Noun>, Option<<Self as _Cell>::Noun>) {
        (self.head, self.tail)
    }
}

impl _IntoNoun for Cell {
    type Error = ();
    type Noun = Noun;

    fn into_noun(self) -> Result<Self::Noun, Self::Error> {
        Ok(Noun::Cell(self))
    }
}

/// We have to use the newtype pattern to implement external traits on external types.
#[derive(Eq, Hash, PartialEq)]
pub struct NounPtr(Box<Noun>);

impl _Cue for NounPtr {
    type Error = ();

    fn cue(jammed_val: Vec<u8>) -> Result<Self, <Self as _Cue>::Error> {
        todo!()
    }
}

impl _Jam for NounPtr {
    type Error = ();

    fn jam(self) -> Result<Vec<u8>, <Self as _Jam>::Error> {
        todo!()
    }
}

impl _Mug for NounPtr {}

impl _Noun for NounPtr {
    type Atom = Atom;
    type Cell = Cell;
    type Error = ();

    fn into_atom(self) -> Result<<Self as _Noun>::Atom, <Self as _Noun>::Error> {
        match *(self.0) {
            Noun::Atom(atom) => Ok(atom),
            _ => Err(()),
        }
    }

    fn into_cell(self) -> Result<<Self as _Noun>::Cell, <Self as _Noun>::Error> {
        match *(self.0) {
            Noun::Cell(cell) => Ok(cell),
            _ => Err(()),
        }
    }
}