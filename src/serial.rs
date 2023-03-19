use std::io::{BufWriter, Cursor};

use eyre::{eyre, Result};
use sucds::serial::Searial;

// helper function for converting sucds structures into
// bytes in a way that plays nice with eyre
pub fn into_bytes<T: Searial>(value: T) -> Result<Vec<u8>> {
    let mut buffer = Vec::<u8>::with_capacity(value.size_in_bytes());
    let writer = BufWriter::new(&mut buffer);
    match value.serialize_into(writer) {
        Ok(_) => Ok(buffer),
        Err(error) => Err(eyre!("{error:?}")),
    }
}

pub fn to_bytes<T: Searial>(value: &T) -> Result<Vec<u8>> {
    let mut buffer = Vec::<u8>::with_capacity(value.size_in_bytes());
    let writer = BufWriter::new(&mut buffer);
    match value.serialize_into(writer) {
        Ok(_) => Ok(buffer),
        Err(error) => Err(eyre!("{error:?}")),
    }
}

pub fn from_bytes<T: Searial>(bytes: Vec<u8>) -> Result<T> {
    let cursor = Cursor::new(bytes);
    match T::deserialize_from(cursor) {
        Ok(store) => Ok(store),
        Err(error) => Err(eyre!("Could not deserialize the BitVector: {error:?}"))?,
    }
}
