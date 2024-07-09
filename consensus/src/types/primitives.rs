use std::ops::Deref;

use hex::encode;
use ssz_rs::prelude::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ByteVector<const N: usize> {
    inner: Vector<u8, N>,
}

impl<const N: usize> ByteVector<N> {
    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }
}

impl<const N: usize> Deref for ByteVector<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

impl<const N: usize> TryFrom<Vec<u8>> for ByteVector<N> {
    type Error = eyre::Report;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: Vector::try_from(value).map_err(|(_, err)| err)?,
        })
    }
}

impl<const N: usize> TryFrom<&[u8]> for ByteVector<N> {
    type Error = eyre::Report;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: Vector::try_from(value.to_vec()).map_err(|(_, err)| err)?,
        })
    }
}

impl<const N: usize> Merkleized for ByteVector<N> {
    fn hash_tree_root(&mut self) -> Result<Node, MerkleizationError> {
        self.inner.hash_tree_root()
    }
}

impl<const N: usize> Sized for ByteVector<N> {
    fn is_variable_size() -> bool {
        false
    }

    fn size_hint() -> usize {
        0
    }
}

impl<const N: usize> Serialize for ByteVector<N> {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        self.inner.serialize(buffer)
    }
}

impl<const N: usize> Deserialize for ByteVector<N> {
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: std::marker::Sized,
    {
        Ok(Self {
            inner: Vector::deserialize(encoding)?,
        })
    }
}

impl<const N: usize> ssz_rs::SimpleSerialize for ByteVector<N> {}

// Added serialize implementation that fits the `deserialize` function (below).
// Default `serde::serialize` implementation wouldn't serialize as expected.
impl<const N: usize> serde::Serialize for ByteVector<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = encode(self.inner.as_slice());
        serializer.serialize_str(format!("0x{}", bytes).as_str())
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for ByteVector<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: String =
            serde::Deserialize::deserialize(deserializer).unwrap_or_else(|_| String::default());
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let bytes = hex::decode(bytes.strip_prefix("0x").unwrap_or_default()).unwrap_or_default();
        Ok(Self {
            inner: bytes.to_vec().try_into().unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct ByteList<const N: usize> {
    inner: List<u8, N>,
}

impl<const N: usize> ByteList<N> {
    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }
}

impl<const N: usize> Deref for ByteList<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

impl<const N: usize> TryFrom<Vec<u8>> for ByteList<N> {
    type Error = eyre::Report;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: List::try_from(value).map_err(|(_, err)| err)?,
        })
    }
}

impl<const N: usize> TryFrom<&[u8]> for ByteList<N> {
    type Error = eyre::Report;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: List::try_from(value.to_vec()).map_err(|(_, err)| err)?,
        })
    }
}

impl<const N: usize> Merkleized for ByteList<N> {
    fn hash_tree_root(&mut self) -> Result<Node, MerkleizationError> {
        self.inner.hash_tree_root()
    }
}

impl<const N: usize> Sized for ByteList<N> {
    fn is_variable_size() -> bool {
        false
    }

    fn size_hint() -> usize {
        0
    }
}

impl<const N: usize> Serialize for ByteList<N> {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        self.inner.serialize(buffer)
    }
}

impl<const N: usize> Deserialize for ByteList<N> {
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: std::marker::Sized,
    {
        Ok(Self {
            inner: List::deserialize(encoding)?,
        })
    }
}

impl<const N: usize> ssz_rs::SimpleSerialize for ByteList<N> {}

impl<'de, const N: usize> serde::Deserialize<'de> for ByteList<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: String = serde::Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(bytes.strip_prefix("0x").unwrap()).unwrap();
        Ok(Self {
            inner: bytes.to_vec().try_into().unwrap(),
        })
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct U64 {
    inner: u64,
}

impl U64 {
    pub fn as_u64(&self) -> u64 {
        self.inner
    }
}

impl From<U64> for u64 {
    fn from(value: U64) -> Self {
        value.inner
    }
}

impl From<u64> for U64 {
    fn from(value: u64) -> Self {
        Self { inner: value }
    }
}

impl Merkleized for U64 {
    fn hash_tree_root(&mut self) -> Result<Node, MerkleizationError> {
        self.inner.hash_tree_root()
    }
}

impl Sized for U64 {
    fn is_variable_size() -> bool {
        false
    }

    fn size_hint() -> usize {
        0
    }
}

impl Serialize for U64 {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        self.inner.serialize(buffer)
    }
}

impl Deserialize for U64 {
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: std::marker::Sized,
    {
        Ok(Self {
            inner: u64::deserialize(encoding)?,
        })
    }
}

impl ssz_rs::SimpleSerialize for U64 {}

impl serde::Serialize for U64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string = self.inner.to_string();
        serializer.serialize_str(string.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for U64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val: String = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self {
            inner: val.parse().unwrap_or_default(),
        })
    }
}
