use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::Div,
    rc::Rc,
    time::{Duration, Instant},
};

use rand::{
    distributions::{Bernoulli, Uniform},
    prelude::Distribution,
    rngs::StdRng,
    SeedableRng,
};
use serde::Serialize;
use sparse_array::{
    math::{div_with_remainder, popcount},
    rank_support::RankSupport,
};
use sucds::BitVector;

#[derive(Debug, Serialize, Clone, Default)]
pub struct FuncCounter {
    pub superblock_count: Duration,
    pub block_count: Duration,
    pub bit_count: Duration,
}

impl FuncCounter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Into<u32>> Div<T> for FuncCounter {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let rhs: u32 = rhs.into();
        Self {
            superblock_count: self.superblock_count / rhs,
            block_count: self.superblock_count / rhs,
            bit_count: self.bit_count / rhs,
        }
    }
}

pub fn profile_rank1(rs: &RankSupport, elem: u64, func_counter: &mut FuncCounter) -> u64 {
    let superblock_position = (elem / rs.s as u64) as usize;
    let (block_position, offset) = div_with_remainder(elem, rs.b as u64);
    let mut now = Instant::now();
    let final_bits =
        unsafe { popcount(rs.store.get_bits((elem - offset) as usize, offset as usize) as u64) }
            as usize;
    func_counter.bit_count += now.elapsed();
    now = Instant::now();
    let block_rank = rs.blocks.get(block_position as usize);
    func_counter.block_count += now.elapsed();
    now = Instant::now();
    let superblock_rank = rs.superblocks.get(superblock_position);
    func_counter.superblock_count += now.elapsed();
    (final_bits + block_rank + superblock_rank) as u64
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Experiment {
    pub fixed: Vec<Run>,
    pub dynamic: Vec<Run>,
}

impl Experiment {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Run {
    counter: FuncCounter,
    param: u64,
}

pub fn execute_run(size: u64, query_size: u64, rng: &mut StdRng, fixed: bool) -> Run {
    let mut func_counter = FuncCounter::default();
    let bits: Vec<bool> = Bernoulli::new(0.5)
        .unwrap()
        .sample_iter(&mut *rng)
        .take(size as usize)
        .collect();
    let store = BitVector::from_bits(bits);
    let rs = match fixed {
        true => RankSupport::with_block_size(32, Rc::new(store)),
        false => RankSupport::new_from_owned(store),
    };

    Uniform::new_inclusive(0, size)
        .sample_iter(&mut *rng)
        .take(query_size as usize)
        .for_each(|elem| {
            profile_rank1(&rs, elem, &mut func_counter);
        });

    // let position_distribution = Uniform::new_inclusive(0, size);
    // let bit_distribution = Bernoulli::new(0.5).unwrap();
    // let store = Rc::new(BitVector::from_bits(
    //     bit_distribution.sample_iter(&mut rng).take(size as usize),
    // ));

    // store.iter().enumerate().for_each(|pair| {
    //     let rank_1 = profile_rank1(&rs, pair.0 as u64, &mut func_counter);
    //     assert_eq!(rank_1, counter);
    //     if pair.1 {
    //         counter += 1;
    //     }
    // });
    func_counter = func_counter / query_size as u32;
    Run {
        counter: func_counter,
        param: size,
    }
}

fn main() {
    let mut experiment = Experiment::default();
    let query_size = 10_000_u64;
    let mut rng = StdRng::from_entropy();
    (1_000_u64..=1_000_000)
        .step_by(10_000)
        .into_iter()
        .for_each(|param| {
            experiment
                .fixed
                .push(execute_run(param, query_size, &mut rng, true));
            experiment
                .dynamic
                .push(execute_run(param, query_size, &mut rng, false))
        });
    println!("Experiment={experiment:#?}");
    let file = File::create("rank-profile.json").unwrap();
    let mut writer = BufWriter::new(file);
    write!(
        &mut writer,
        "{}",
        serde_json::to_string(&experiment).unwrap()
    )
    .unwrap()
    // println!("{}", serde_json::to_string(&experiment).unwrap());
    // println!("RS = {rs:?}");
    // let sparse_array = SparseArray::from_dense(vec![Some(1), Some(2), None, None, Some(3), None]);
    // println!("The original array = {sparse_array:?}");
    // sparse_array.save_into("sparse_array.bin").unwrap();
    // let new_array = SparseArray::<i32>::load("sparse_array.bin").unwrap();
    // println!("The loaded array = {new_array:?}");
}
