use crate::{
    math::{ceil_div, div_with_remainder, log2_ceil, popcount},
    serial::{from_bytes, into_bytes, to_bytes},
};
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    cmp::{max, min},
    fs::File,
    io::{BufReader, BufWriter},
    rc::Rc,
};
use std::mem::size_of;
use sucds::{BitVector, CompactVector, Searial};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveableRankSupport {
    pub store: Vec<u8>,
    pub superblocks: Vec<u8>,
    pub blocks: Vec<u8>,
    pub s: u16,
    pub b: u8,
}

impl TryFrom<RankSupport> for SaveableRankSupport {
    type Error = eyre::Report;
    fn try_from(value: RankSupport) -> eyre::Result<Self> {
        let superblocks = into_bytes(value.superblocks)?;
        let blocks = into_bytes(value.blocks)?;
        let store = to_bytes(value.store.as_ref())?;
        Ok(Self {
            store,
            superblocks,
            blocks,
            b: value.b,
            s: value.s,
        })
    }
}

impl TryFrom<SaveableRankSupport> for RankSupport {
    type Error = eyre::Report;
    fn try_from(value: SaveableRankSupport) -> Result<Self, Self::Error> {
        let store: BitVector = from_bytes(value.store)?;
        let superblocks: CompactVector = from_bytes(value.superblocks)?;
        let blocks: CompactVector = from_bytes(value.blocks)?;
        Ok(Self {
            store: Rc::new(store),
            superblocks,
            blocks,
            s: value.s,
            b: value.b,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "SaveableRankSupport")]
pub struct RankSupport {
    pub store: Rc<BitVector>,

    // overhead
    pub superblocks: CompactVector,
    pub blocks: CompactVector,
    pub b: u8,
    pub s: u16,
}

impl RankSupport {
    pub fn with_block_size(block_size: u64, store: Rc<BitVector>) -> Self {
        let n = store.len() as u64;
        let log2_n = max(log2_ceil(n), 1);
        let log2_n_squared = log2_n * log2_n;

        //let blocks_per_superblock = ceil_div(ceil_div(log2_n_squared, 2), block_size);
        let blocks_per_superblock = ceil_div(log2_n_squared, block_size);
        let superblock_size = block_size * blocks_per_superblock;

        // not using ceil_div here because we want there to always be one more superblock than we need
        let number_of_superblocks = (n / superblock_size) + 1;
        // let number_of_superblocks = ceil_div(n, superblock_size);
        let number_of_blocks = blocks_per_superblock * number_of_superblocks;

        let superblock_rank_width = log2_ceil(n + 1);
        let block_rank_width = log2_ceil(superblock_size);

        let mut superblock_cumulative_ranks = CompactVector::with_capacity(
            number_of_superblocks as usize,
            superblock_rank_width as usize,
        );
        let mut block_cumulative_ranks =
            CompactVector::with_capacity(number_of_blocks as usize, block_rank_width as usize);

        let mut cumulative_rank = 0_usize;
        let mut previous_cumulative_rank = 0_usize;
        let mut position = 0_usize;

        for block_idx in 0..number_of_blocks {
            if block_idx % blocks_per_superblock == 0 {
                superblock_cumulative_ranks.push(cumulative_rank);
                previous_cumulative_rank = cumulative_rank;
            }
            block_cumulative_ranks.push(cumulative_rank - previous_cumulative_rank);
            let block_len = min(block_size, n - position as u64) as usize;
            cumulative_rank +=
                unsafe { popcount(store.get_bits(position, block_len) as u64) } as usize;
            // cumulative_rank += store.get_bits(position, block_len).count_ones() as usize;
            position += block_len;
        }

        Self {
            store,
            superblocks: superblock_cumulative_ranks,
            blocks: block_cumulative_ranks,
            s: superblock_size as u16,
            b: block_size as u8,
        }
    }

    pub fn new(store: Rc<BitVector>) -> Self {
        let n = store.len() as u64;
        let log2_n = max(log2_ceil(n), 1);

        let block_size = ceil_div(log2_n, 2);
        Self::with_block_size(block_size, store)
    }

    pub fn new_from_owned(store: BitVector) -> Self {
        Self::new(Rc::new(store))
    }

    pub fn rank1(&self, elem: u64) -> u64 {
        let superblock_position = (elem / self.s as u64) as usize;
        let (block_position, offset) = div_with_remainder(elem, self.b as u64);
        let final_bits = unsafe {
            popcount(
                self.store
                    .get_bits((elem - offset) as usize, offset as usize) as u64,
            )
        } as usize;
        (self.superblocks.get(superblock_position)
            + self.blocks.get(block_position as usize)
            + final_bits) as u64
    }

    pub fn rank0(&self, elem: u64) -> u64 {
        elem - self.rank1(elem)
    }

    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let saveable: SaveableRankSupport = self.try_into()?;
        bincode::serialize(&saveable).wrap_err("Failed to serialize rank support")
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.clone().into_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).wrap_err("Failed to deserialize rank_support")
    }

    /// The size in bits required to support constant time rank queries
    pub fn overhead(&self) -> u64 {
        (self.blocks.size_in_bytes()
            + self.superblocks.size_in_bytes()
            + self.s.size_in_bytes()
            + self.b.size_in_bytes()
            + size_of::<Rc<BitVector>> as usize
        ) as u64 * 8
    }

    pub fn save(&self, fname: &str) -> Result<()> {
        let file = File::create(fname).wrap_err(format!("Failed to create file {fname}"))?;
        let mut writer = BufWriter::new(file);
        let clone: SaveableRankSupport = self.clone().try_into()?;
        bincode::serialize_into(&mut writer, &clone)?;
        Ok(())
    }

    pub fn load(fname: &str) -> Result<Self> {
        let file = File::open(fname).wrap_err(format!("Failed to open file {fname}"))?;
        let reader = BufReader::new(file);
        let result: RankSupport = bincode::deserialize_from(reader)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Bernoulli, prelude::Distribution, rngs::StdRng, SeedableRng};
    use sucds::BitVector;

    #[test]
    fn test_small_bitvectors() {
        for i in 0..2_u64 {
            let bv = BitVector::from_bits(vec![true; i as usize]);
            let rs = RankSupport::new_from_owned(bv);
            assert_eq!(i, rs.rank1(i));
            assert_eq!(0, rs.rank0(i));
        }
    }

    #[test]
    fn test_rank1() {
        let bv = BitVector::from_bits([false, true, true, true, false]);
        let rs = RankSupport::new_from_owned(bv);
        assert_eq!(0, rs.rank1(0));
        assert_eq!(0, rs.rank1(1));
        assert_eq!(1, rs.rank1(2));
        assert_eq!(2, rs.rank1(3));
        assert_eq!(3, rs.rank1(4));
    }

    #[test]
    fn test_rank0() {
        let bv = BitVector::from_bits([false, true, true, true, false]);
        let rs = RankSupport::new_from_owned(bv);
        assert_eq!(0, rs.rank0(0));
        assert_eq!(1, rs.rank0(1));
        assert_eq!(1, rs.rank0(2));
        assert_eq!(1, rs.rank0(3));
        assert_eq!(1, rs.rank0(4));
        assert_eq!(2, rs.rank0(5));
    }

    #[test]
    fn test_off_by_one() {
        let bv = BitVector::from_bits([true, false, false, true]);
        let rs = RankSupport::new_from_owned(bv);
        assert_eq!(1, rs.rank1(3));
        assert_eq!(2, rs.rank1(4));
    }

    #[test]
    fn test_various_sizes() {
        let mut rng = StdRng::from_entropy();
        let distribution = Bernoulli::new(0.5).unwrap();
        (10_000..10_128_u64).for_each(|size| {
            let bits = distribution
                .sample_iter(&mut rng)
                .take(size as usize)
                .collect::<Vec<bool>>();
            let mut expected_ranks = Vec::<u64>::with_capacity(size as usize);
            let mut counter = 0_u64;
            bits.iter().for_each(|&value| {
                expected_ranks.push(*&counter);
                if value {
                    counter += 1
                }
            });
            expected_ranks.push(counter);
            let rs = RankSupport::new_from_owned(BitVector::from_bits(bits));
            expected_ranks
                .into_iter()
                .enumerate()
                .for_each(|(pos, rank)| {
                    let result = rs.rank1(pos as u64);
                    assert_eq!(result, rank);
                })
        })
    }
}
