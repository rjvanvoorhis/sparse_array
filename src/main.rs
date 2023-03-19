use std::rc::Rc;

use sparse_array::{rank_support::RankSupport, sparse_array::SparseArray};
use sucds::{BitVector, Searial};

fn main() {
    let bit_vec = BitVector::from_bits(vec![true; 8 * 1000000000]);
    let rs = RankSupport::new(Rc::new(bit_vec));
    let actual_overhead = rs.overhead();
    let theoretical_overhead = rs.theoretical_overhead();
    let actual_bv_size = rs.store.size_in_bytes();
    let theoretical_bv_size = rs.store.len() / 8;

    // println!("RS = {rs:?}");
    println!("BV Size (actual) = {actual_bv_size}; (theoretical) = {theoretical_bv_size}");
    println!("RS Size (actual) = {actual_overhead}; (theoretical) = {theoretical_overhead}");
    println!(
        "Overhead PCT (actual) = {}; (theoretical) = {}",
        actual_overhead as f64 / actual_bv_size as f64,
        theoretical_overhead as f64 / theoretical_bv_size as f64
    );
    // let sparse_array = SparseArray::from_dense(vec![Some(1), Some(2), None, None, Some(3), None]);
    // println!("The original array = {sparse_array:?}");
    // sparse_array.save_into("sparse_array.bin").unwrap();
    // let new_array = SparseArray::<i32>::load("sparse_array.bin").unwrap();
    // println!("The loaded array = {new_array:?}");
}
