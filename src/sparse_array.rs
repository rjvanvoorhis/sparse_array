use std::{
    fs::File,
    io::{BufReader, BufWriter},
    rc::Rc,
};

use eyre::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sucds::{BitVector, Searial};

use crate::{rank_support::RankSupport, select_support::SelectSupport};

#[derive(Debug, Clone)]
pub struct SparseArray<T> {
    vector: Vec<T>,
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
        SparseArray {
            vector: self.vector,
            rank_support,
            select_support,
        }
    }
}

impl<T: Serialize + Clone + DeserializeOwned> SparseArray<T> {
    // Generate a static SparseArray from parts
    pub fn new(vector: Vec<T>, store: BitVector) -> Self {
        SparseArrayBuilder { vector, store }.finalize()
    }

    // Create an empty SparseArrayBuilder which elements can be added to
    pub fn create(size: u64) -> SparseArrayBuilder<T> {
        SparseArrayBuilder::new(size)
    }

    pub fn overhead(&self) -> u64 {
        (self.rank_support.store.size_in_bytes() as u64 * 8) + 
        self.select_support.overhead() + 64
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
    pub fn from_dense_vec(values: Vec<Option<T>>) -> Self {
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

    /// create a sparse array from an iterable of optional values
    ///
    /// ```
    /// # use sparse_array::sparse_array::*;
    /// let some_if_even = |x: u32| {if x % 2 == 0 {Some(x)} else {None}};
    /// let values = (0..=10_u32).map(some_if_even);
    /// let sparse = SparseArray::from_dense(values);
    /// assert_eq!(0, *sparse.get_at_index(0).unwrap());
    /// assert_eq!(None, sparse.get_at_index(1));
    /// assert_eq!(2, *sparse.get_at_index(2).unwrap());
    /// ```
    pub fn from_dense<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Option<T>>,
    {
        let values: Vec<Option<T>> = values.into_iter().collect();
        Self::from_dense_vec(values)
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
        if index >= self.size() {
            return None;
        }
        match self.rank_support.store.get_bit(index as usize) {
            true => self.get_at_rank(self.rank_support.rank1(index)),
            false => None,
        }
    }

    /// Given a target rank return the index of the element in the sparse array where the rankth element occurs
    ///
    /// If rank > num_elem() returns None
    ///
    /// ```
    /// # use sparse_array::sparse_array::*;
    /// let sparse = SparseArray::from_dense(vec![None, Some(0), Some(1), None, None, Some(2)]);
    /// assert_eq!(None, sparse.get_index_of(0));
    /// assert_eq!(1, sparse.get_index_of(1).unwrap());
    /// assert_eq!(2, sparse.get_index_of(2).unwrap());
    /// assert_eq!(5, sparse.get_index_of(3).unwrap());
    /// ```
    pub fn get_index_of(&self, rank: u64) -> Option<u64> {
        let num_elem = self.num_elem();
        if rank > num_elem || rank == 0 {
            return None;
        }
        if rank == 0 {
            return Some(0);
        }

        Some(self.select_support.select1(rank) - 1)
    }

    pub fn load(fname: &str) -> Result<Self> {
        let file = File::open(fname)?;
        let reader = BufReader::new(file);
        let interim_sparse_array: InterimSparseArray<T> = bincode::deserialize_from(reader)?;
        interim_sparse_array.try_into()
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, SeedableRng};

    #[test]
    fn test_save_load(){
        let sa = SparseArray::<u32>::from_dense_vec(vec![None, Some(1), None, Some(2), Some(3)]);
        assert_eq!(*sa.get_at_index(1).unwrap(), 1);
        assert_eq!(sa.get_index_of(1).unwrap(), 1);
        assert_eq!(sa.num_elem_at(1), 1);
        sa.save("tmp-file.bin").unwrap();
        let loaded: SparseArray<u32> = SparseArray::load("tmp-file.bin").unwrap();
        assert_eq!(*loaded.get_at_index(1).unwrap(), 1);
        assert_eq!(loaded.get_index_of(1).unwrap(), 1);
        assert_eq!(loaded.num_elem_at(1), 1);
    }

    #[test]
    fn test_from_dense_vec() {
        let distribution = Uniform::new_inclusive(0, 100_u8);
        let mut rng = StdRng::seed_from_u64(42);
        let dense: Vec<Option<u32>> = distribution
            .sample_iter(&mut rng)
            .take(10000)
            .enumerate()
            .map(
                |(index, roll)| {
                    if roll < 15 {
                        Some(index as u32)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let sparse = SparseArray::<u32>::from_dense(dense.clone());
        dense
            .into_iter()
            .enumerate()
            .for_each(|(index, value)| match value {
                None => assert_eq!(None, sparse.get_at_index(index as u64)),
                Some(inner) => {
                    assert_eq!(inner, sparse.get_at_index(index as u64).unwrap().to_owned())
                }
            })
    }

    #[test]
    fn test_get_index_of() {
        let length = 10000_u64;
        for sparsity in (0..=100_u8).step_by(5) {
            let mut rng = StdRng::seed_from_u64(42);
            let distibution = Uniform::new_inclusive(0, 100_u8);
            let mut sparse = SparseArray::<u64>::create(length);
            let mut expected_positions = Vec::<u64>::new();
            distibution
                .sample_iter(&mut rng)
                .take(length as usize)
                .enumerate()
                .filter_map(|pair| {
                    if pair.1 < sparsity {
                        Some(pair.0 as u64)
                    } else {
                        None
                    }
                })
                .for_each(|pos| {
                    sparse.append(pos, pos);
                    expected_positions.push(pos);
                });
            let sparse = sparse.finalize();
            assert_eq!(
                None,
                sparse.get_index_of(expected_positions.len() as u64 + 1)
            );
            expected_positions.into_iter().enumerate().for_each(
                |(rank_minus_1, expected_position)| {
                    assert_eq!(
                        expected_position,
                        sparse.get_index_of(rank_minus_1 as u64 + 1).unwrap()
                    );
                },
            );
        }
    }
}
