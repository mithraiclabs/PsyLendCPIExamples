use std::{ops::{Deref, DerefMut}, fmt::Display};

use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use psy_math::Number;

/// A fixed-size byte array
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FixedBuf<const SIZE: usize> {
    data: [u8; SIZE],
}
unsafe impl<const SIZE: usize> Pod for FixedBuf<SIZE> {}
unsafe impl<const SIZE: usize> Zeroable for FixedBuf<SIZE> {}
impl<const SIZE: usize> std::fmt::Debug for FixedBuf<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FixedBuf<{}>", SIZE)
    }
}
impl<const SIZE: usize> AsRef<[u8]> for FixedBuf<SIZE> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}
impl<const SIZE: usize> AsMut<[u8]> for FixedBuf<SIZE> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}
impl<const SIZE: usize> Deref for FixedBuf<SIZE> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<const SIZE: usize> DerefMut for FixedBuf<SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl<const SIZE: usize> borsh::BorshDeserialize for FixedBuf<SIZE> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let mut data = [0u8; SIZE];
        data.copy_from_slice(buf);

        Ok(FixedBuf { data })
    }
}
impl<const SIZE: usize> borsh::BorshSerialize for FixedBuf<SIZE> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let _ = writer.write(&self.data)?;
        Ok(())
    }
}


/// Workaround for the fact that `Pubkey` doesn't implement the
/// `Pod` trait (even though it meets the requirements), and there
/// isn't really a way for us to extend the original type, so we wrap
/// it in a new one.
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(transparent)]
pub struct StoredPubkey(Pubkey);

impl AsRef<Pubkey> for StoredPubkey {
    fn as_ref(&self) -> &Pubkey {
        &self.0
    }
}
impl From<StoredPubkey> for Pubkey {
    fn from(key: StoredPubkey) -> Self {
        key.0
    }
}
impl From<Pubkey> for StoredPubkey {
    fn from(key: Pubkey) -> Self {
        Self(key)
    }
}
impl std::fmt::Debug for StoredPubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self.0 as &dyn std::fmt::Display).fmt(f)
    }
}
impl Display for StoredPubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl PartialEq<Pubkey> for StoredPubkey {
    fn eq(&self, other: &Pubkey) -> bool {
        self.0.eq(other)
    }
}
unsafe impl Pod for StoredPubkey {}
unsafe impl Zeroable for StoredPubkey {}

/// Linear interpolation between (x0, y0) and (x1, y1).
pub fn interpolate(x: Number, x0: Number, x1: Number, y0: Number, y1: Number) -> Number {
    assert!(x >= x0);
    assert!(x <= x1);

    y0 + ((x - x0) * (y1 - y0)) / (x1 - x0)
}