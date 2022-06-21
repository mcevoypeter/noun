use crate::{Atom, Cell, Noun};
use bitstream_io::{BitRead, BitWrite};
use std::{
    collections::HashMap,
    fmt::Debug,
    mem::{drop, size_of},
    rc::Rc,
};

/// (<some type>, bits read)
#[doc(hidden)]
pub type CueResult<T> = Result<(T, u32), ()>;

/// Deserialize a bitstream into a noun.
pub trait Cue<A, C>
where
    A: Atom<C, Self>,
    C: Cell<A, Self>,
    Self: Noun<A, C> + Debug + Sized,
{
    /// Decodes a bitstream into a noun.
    ///
    /// The bitstream is read from least significant bit to most significant bit and starts with a
    /// tag identifying whether the object following the tag is an atom, a cell, or a backreference
    /// to an object that was already decoded. The tag encodings are:
    /// - `0b0`: atom,
    /// - `0b01`: cell, and
    /// - `0b11`: backreference.
    ///
    /// Note that the tag for an atom is only a single bit whereas the tags for a cell and a
    /// backreference are both two bits.
    fn cue(mut src: impl BitRead) -> Result<Self, ()> {
        let mut cache = HashMap::new();
        let (noun, _) = Self::decode(&mut src, &mut cache, 0)?;

        // Dropping the cache guarantees that the top level noun has exactly one reference, which
        // makes it safe to move out of the Rc.
        drop(cache);
        let noun = Rc::try_unwrap(noun).unwrap();

        Ok(noun)
    }

    #[doc(hidden)]
    fn decode(
        src: &mut impl BitRead,
        cache: &mut HashMap<u64, Rc<Self>>,
        pos: u64,
    ) -> CueResult<Rc<Self>> {
        match src.read_bit() {
            Ok(true) => {
                const TAG_LEN: u32 = 2;
                match src.read_bit() {
                    // Back reference tag = 0b11.
                    Ok(true) => {
                        let (noun, bits_read) = Self::decode_backref(src, cache, pos)?;
                        Ok((noun, TAG_LEN + bits_read))
                    }
                    // Cell tag = 0b01.
                    Ok(false) => {
                        let (cell, bits_read) = Self::decode_cell(src, cache, pos)?;
                        Ok((cell, TAG_LEN + bits_read))
                    }
                    Err(_) => todo!("IO error"),
                }
            }
            // Atom tag = 0b0.
            Ok(false) => {
                const TAG_LEN: u32 = 1;
                let (atom, bits_read) = Self::decode_atom(src, Some(cache), pos)?;
                Ok((atom, TAG_LEN + bits_read))
            }
            Err(_) => {
                todo!("I think this is when it's time to exit")
            }
        }
    }

    /// Decode the length of an atom or backreference.
    #[doc(hidden)]
    fn decode_len(src: &mut impl BitRead) -> CueResult<u64> {
        let len_of_len = src.read_unary0().expect("count high bits");
        // Length must be 63 bits or less.
        if len_of_len >= u64::BITS {
            todo!("too large")
        }

        let len: u64 = src.read(len_of_len).expect("get length");
        // Most significant bit of the length is always one and always omitted, so add it back now.
        let len = (1 << len_of_len) | len;

        let bits_read = 2 * len_of_len + 1;
        Ok((len, bits_read))
    }

    /// Decode an encoded atom from the bitstream. Note that the atom tag must already be consumed,
    /// which means that the first bit read from `src` (located at index `pos`) is the first bit of
    /// the encoded length.
    #[doc(hidden)]
    fn decode_atom(
        src: &mut impl BitRead,
        cache: Option<&mut HashMap<u64, Rc<Self>>>,
        pos: u64,
    ) -> CueResult<Rc<Self>> {
        // Decode the atom length.
        let (mut bit_len, mut bits_read) = Self::decode_len(src)?;

        let mut val = {
            // This will allocate an extra byte when bit_len is a multiple of u8::BITS, but it's
            // worth it to omit a branch.
            let byte_len = (bit_len / u64::from(u8::BITS)) + 1;
            let byte_len = usize::try_from(byte_len).expect("u64 doesn't fit in usize");
            Vec::with_capacity(byte_len)
        };
        while bit_len > u64::from(u8::BITS) {
            let byte: u8 = src.read(u8::BITS).expect("read chunk");
            bits_read += u8::BITS;
            val.push(byte);
            bit_len -= u64::from(u8::BITS);
        }
        // Consume remaining bits.
        let bit_len = u32::try_from(bit_len).unwrap();
        let byte: u8 = src.read(bit_len).expect("read chunk");
        bits_read += bit_len;
        val.push(byte);

        let atom = Rc::new(A::from(val).into_noun().unwrap());
        if let Some(cache) = cache {
            cache.insert(pos, atom.clone());
        }

        Ok((atom, bits_read))
    }

    /// Decode an encoded backreference from the bitstream. Note that the backreference tag must
    /// already be consumed, which means that the first bit read from `src` (located at index
    /// `pos`) is the first bit of the encoded length.
    #[doc(hidden)]
    fn decode_backref(
        src: &mut impl BitRead,
        cache: &mut HashMap<u64, Rc<Self>>,
        pos: u64,
    ) -> CueResult<Rc<Self>> {
        let (idx, bits_read) = Self::decode_atom(src, None, pos)?;
        let (first, rest) = idx.as_atom()?.as_bytes().split_at(size_of::<u64>());
        if rest.len() > 0 {
            todo!("idx is larger than 8 bytes")
        }
        // XXX: watch out for endianness bug.
        let idx = u64::from_le_bytes(first.try_into().unwrap());
        if let Some(noun) = cache.get(&idx) {
            Ok((noun.clone(), bits_read))
        } else {
            Err(())
        }
    }

    /// Decode a cell from the bitstream. Note that the cell tag must already be consumed, which
    /// means that the first bit read from `src` (located at index `pos`) is the first bit of the
    /// head's tag.
    #[doc(hidden)]
    fn decode_cell(
        src: &mut impl BitRead,
        cache: &mut HashMap<u64, Rc<Self>>,
        mut pos: u64,
    ) -> CueResult<Rc<Self>> {
        let (head, head_bits) = Self::decode(src, cache, pos)?;
        cache.insert(pos, head.clone());

        pos += u64::from(head_bits);

        let (tail, tail_bits) = Self::decode(src, cache, pos)?;
        cache.insert(pos, tail.clone());

        let cell = Rc::new(Self::new_cell(head, tail).into_noun().unwrap());
        Ok((cell, head_bits + tail_bits))
    }

    /// Construct a new cell.
    ///
    /// The construction of a cell cannot be generalized using the `Cell` trait for use in this
    /// context because the `Cell::Head` and `Cell::Tail` traits are intentionally not bounded by
    /// the `Noun` trait, which would be too onerous on implementers. Beside cell construction,
    /// cueing (decoding) a jammed (encoded) noun is completely independent of the noun
    /// representation, so deserializing a serialized noun is completely independent of the noun
    /// representation, so implementing this single method on a particular noun type will result in
    /// a free implementation of cue.
    fn new_cell(head: Rc<Self>, tail: Rc<Self>) -> C;
}

/// (<some type>, bits read)
#[doc(hidden)]
pub type JamResult<T> = Result<(T, u32), ()>;

/// Serialize a noun into a bitstream.
pub trait Jam<A, C>
where
    A: Atom<C, Self>,
    C: Cell<A, Self>,
    Self: Noun<A, C> + Sized,
{
    fn jam(self, sink: &mut impl BitWrite) -> Result<(), ()>;
}
