#![cfg_attr(not(feature = "std"), no_std)]
// #![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::declare_interior_mutable_const)]
#![allow(clippy::borrow_interior_mutable_const)]
extern crate alloc;
mod smt;
pub use self::smt::SparseMerkleTree;

mod proofs;
mod tree_hasher;

pub use bytes;
use bytes::Bytes;
pub use digest;
pub use proofs::*;

/// Key-Value store
pub trait KVStore {
    /// The hasher to use for the underlying tree.
    type Hasher: digest::Digest;

    /// The Error type
    #[cfg(not(feature = "std"))]
    type Error: core::fmt::Debug + core::fmt::Display + From<BadProof>;

    /// The Error type
    #[cfg(feature = "std")]
    type Error: std::error::Error + From<BadProof>;

    /// Gets the value for a key. If not exists, returns `Ok(None)`.
    fn get(&self, key: &[u8]) -> Result<Option<Bytes>, Self::Error>;
    /// Updates the value for a key.
    fn set(&mut self, key: Bytes, value: Bytes) -> Result<(), Self::Error>;
    /// Remove value by key.
    fn remove(&mut self, key: &[u8]) -> Result<Bytes, Self::Error>;
    /// Returns if key exists in the store.
    fn contains(&self, key: &[u8]) -> Result<bool, Self::Error>;

    fn pin(&mut self);
    fn rollback(&mut self);
    fn finalize(&self);
}

/// Gets the bit at an offset from the most significant bit
#[inline]
fn get_bit_at_from_msb(data: &[u8], position: usize) -> usize {
    if (data[position / 8] as usize) & (1 << (8 - 1 - (position % 8))) > 0 {
        return 1;
    }
    0
}

/// Sets the bit at an offset from the most significant bit
#[inline]
fn set_bit_at_from_msb(data: &mut [u8], position: usize) {
    let mut n = data[position / 8] as usize;
    n |= 1 << (8 - 1 - (position % 8));
    data[position / 8] = n as u8;
}

#[inline]
fn count_set_bits(data: &[u8]) -> usize {
    let mut count = 0;
    for i in 0..data.len() * 8 {
        if get_bit_at_from_msb(data, i) == 1 {
            count += 1;
        }
    }
    count
}

#[inline]
fn count_common_prefix(a: &[u8], b: &[u8]) -> usize {
    let mut cnt = 0;
    for i in 0..a.len() * 8 {
        if get_bit_at_from_msb(a, i) == get_bit_at_from_msb(b, i) {
            cnt += 1;
            continue;
        }
        break;
    }
    cnt
}
