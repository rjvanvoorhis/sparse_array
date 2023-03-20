use std::{
    fs::File,
    io::{BufReader, BufWriter},
    rc::Rc,
};

use eyre::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sucds::BitVector;

use crate::{rank_support::RankSupport, select_support::SelectSupport};

#[derive(Debug, Clone)]
pub struct SparseArray<T> {
    vector: Vec<T>,
    // store: Rc<BitVector>,
    rank_support: Rc<RankSupport>,
    select_support: Rc<SelectSupport>,
}

#[derive(Debug)]
pub struct SparseArrayBuilder<T> {
    vector: Vec<T>,
    store: BitVector,
}

#[derive(Serialize, Deserialize)]
struct InterimSparseArray<T> {
    pub rank_support_bytes: Vec<u8>,
    pub vector: Vec<T>,
}

impl<T: Serialize + Clone> TryFrom<InterimSparseArray<T>> for SparseArray<T> {
    type Error = eyre::Report;

    fn try_from(value: InterimSparseArray<T>) -> std::result::Result<Self, Self::Error> {
        let inner_rank_support = RankSupport::from_bytes(&value.rank_support_bytes)?;
        let rank_support = Rc::new(inner_rank_support);
        let inner_select_support = SelectSupport::new(Rc::clone(&rank_support));
        let select_support = Rc::new(inner_select_support);
        Ok(Self {
            vector: value.vector,
            rank_support,
            select_support,
        })
    }
}

impl<T: Serialize + Clone + DeserializeOwned> SparseArrayBuilder<T> {
    pub fn new(size: u64) -> Self {
        Self {
            vector: Vec::with_capacity(size as usize),
            store: BitVector::from_bits(vec![false; size as usize]),
        }
    }

    pub fn append(&mut self, value: T, pos: u64) {
        self.vector.push(value);
        self.store.set_bit(pos as usize, true);
    }

    /// Build all support structures and return final locked sparse array
    pub fn finalize(self) -> SparseArray<T> {
        let store = Rc::new(self.store);
        let inner_rank_support = RankSupport::new(store);
        let rank_support = Rc::new(inner_rank_support);
        let inner_select_support = SelectSupport::new(Rc::clone(&rank_support));
        let select_support = Rc::new(inner_select_support);
        return SparseArray {
            vector: self.vector,
            rank_support,
            select_support,
        };
    }
}

impl<T: Serialize + Clone + DeserializeOwned> SparseArray<T> {
    pub fn new(size: u64) -> SparseArrayBuilder<T> {
        SparseArrayBuilder::new(size)
    }

    /// An alias for new
    pub fn create(size: u64) -> SparseArrayBuilder<T> {
        Self::new(size)
    }

    /// create a sparse array from a dense vector
    ///
    /// ```
    /// # use sparse_array::sparse_array::*;
    /// let dense = vec![Some(1_i32), None, None, Some(4), None];
    /// let sparse = SparseArray::from_dense(dense.clone());
    /// let from_dense = dense.get(0).unwrap().unwrap();
    /// let from_sparse = *sparse.get_at_index(0).unwrap();
    /// # assert_eq!(from_dense, 1);
    /// assert_eq!(from_dense, from_sparse);
    /// assert_eq!(sparse.get_at_index(1), None);
    /// ```
    pub fn from_dense<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Option<T>> + ExactSizeIterator,
    {
        let mut builder = Self::create(values.len() as u64);
        values.into_iter().enumerate().for_each(|(pos, value)| {
            match value {
                None => {}
                Some(inner) => {
                    builder.append(inner, pos as u64);
                }
            };
        });
        builder.finalize()
    }

    /// Returns the total number elements in the dense representation of the array
    ///
    /// ```
    /// # use sparse_array::sparse_array::*;
    /// let sparse = SparseArray::from_dense(vec![Some(1_i32), Some(2), None, Some(3), None]);
    /// assert_eq!(sparse.size(), 5);
    /// ```
    pub fn size(&self) -> u64 {
        self.rank_support.store.len() as u64
    }

    /// Returns the number of elements that have a value
    ///
    /// ```
    /// # use sparse_array::sparse_array::*;
    /// let sparse = SparseArray::from_dense(vec![Some(1_i32), Some(2), None, Some(3), None]);
    /// assert_eq!(sparse.num_elem(), 3);
    /// ```
    pub fn num_elem(&self) -> u64 {
        self.vector.len() as u64
    }

    pub fn num_elem_at(&self, index: u64) -> u64 {
        self.rank_support.rank1(index + 1)
    }

    pub fn get_at_rank(&self, rank: u64) -> Option<&T> {
        self.vector.get(rank as usize)
    }

    pub fn get_at_index(&self, index: u64) -> Option<&T> {
        match self.rank_support.store.get_bit(index as usize) {
            true => self.get_at_rank(self.rank_support.rank1(index)),
            false => None,
        }
    }

    pub fn get_index_of(&self, rank: u64) -> Option<u64> {
        if rank > self.num_elem() {
            None
        } else {
            Some(self.select_support.select1(rank) - 1)
        }
    }

    pub fn load(fname: &str) -> Result<Self> {
        let file = File::open(fname)?;
        let reader = BufReader::new(file);
        let interim_sparse_array: InterimSparseArray<T> = bincode::deserialize_from(reader)?;
        Ok(interim_sparse_array.try_into()?)
    }

    pub fn save(&self, fname: &str) -> Result<()> {
        let interim_sparse_array = InterimSparseArray {
            vector: self.vector.clone(),
            rank_support_bytes: self.rank_support.to_bytes()?,
        };
        let file = File::create(fname)?;
        let mut writer = BufWriter::new(file);
        bincode::serialize_into(&mut writer, &interim_sparse_array)
            .wrap_err("Failed to serialize sparse array")?;
        Ok(())
    }

    pub fn save_into(self, fname: &str) -> Result<()> {
        let interim_sparse_array = InterimSparseArray {
            vector: self.vector,
            rank_support_bytes: self.rank_support.to_bytes()?,
        };
        let file = File::create(fname)?;
        let mut writer = BufWriter::new(file);
        bincode::serialize_into(&mut writer, &interim_sparse_array)
            .wrap_err("Failed to serialize sparse array")?;
        Ok(())
    }
}
