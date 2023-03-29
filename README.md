# Homework 2: Sparse Array

Author: Ryan Van Voorhis
Github: https://github.com/rjvanvoorhis/sparse_array

## 3rd party libraries

- sucds: Builds succinct datastructures including bitvector and intvectors
- serde + bincode: Used serialization in save/load methods
- clap: command line argument parser for experiments
- eyre: error handling

## Section 1: Rank Support

The rank support struct implements Jacobson rank for constant time rank queries.

The rank support struct can be initialized via a reference counted pointer to a bitvector

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new(Rc::new(bv));
```

or from a bitvector passed by value

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new_from_owned(bv);
```

rank queries may be called with the following methods;

- rs.rank1(elem)
- rs.rank0(elem)

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new_from_owned(bv);
// get the number of ones set in bv up to but not including position 1 => 1
assert_eq!(1, rs.rank_1(1));

// get the number of zeros set in bv up to but not including position 1 => 0
assert_eq!(0, rs.rank0(1));
```

To get the amount of memory the auxiliary rank support structures consume a `rs.overhead()` method is provided. The memory usage is measured in bits and is represented by a u64 integer.

The rank support structure may be saved and loaded from disk using the `save` and `load` methods:

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new_from_owned(bv);
rs.save("tmp.bin").unwrap();
let rs_loaded: RankSupport = RankSupport::load("tmp.bin").unwrap();
assert_eq!(rs_loaded.rank1(1), 1);
```

## Section 2: Select Support

The select support struct wraps RankSupport much the same way RankSupport wraps the bitvector. It can be initialized in much the same way. Either by value or via an Rc

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new_from_owned(bv);
let s = SelectSupport::new_from_owned(bv);
```

or

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new_from_owned(bv);
let s = SelectSupport::new(SelectSupport::new(Rc::new(rs)));
```

The struct supports the following queries:

- select1(i: u64) -> u64: returns the first integer j such that rank1(j) == i
- select0(i: u64) -> u64: returns the first integer j such that rank0(j) == i

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new_from_owned(bv);
let s = SelectSupport::new(SelectSupport::new(Rc::new(rs)));
assert_eq!(s.select1(1), 1);
assert_eq!(s.select0(1), 2);
```

Similar to RankSupport, SelectSupport provides an `overhead()` method that returns the number of bits required to support select queries. This is simply the overhead of the RankSupport struct plus the size of the Rc that wraps it.

Finally the select support may be saved and loaded from disk

```rust
let bv = BitVector::from_bits([true, false, true]);
let rs = RankSupport::new_from_owned(bv);
let s = SelectSupport::new_from_owned(rs);
s.save("select.bin").unwrap();
let s_loaded = SelectSupport::load("select.bin").unwrap();
assert_eq!(s_loaded.select1(1), 1);
assert_eq!(s_loaded.select0(1), 2);
```

## Section 3: Sparse Array

The sparse array is built on top of the previous two structures. It uses a builder patter to construct the sparse array.

The sparse array can be created by first specifying a type and a length of the array.

```rust
let builder = SparseArray<String>::create(7);
// this prepares a sparse array of Strings 10 elements long.
// builder is of type SparseArrayBuilder<String>
```

Elements can that be added to the array at specified positions

```rust
builder.append(String::from("foo"), 1);
builder.append(String::from("bar"), 3);
```

Then the sparse array is locked and the rank support structures built by calling the finalize method.

```rust
let sparse_array = builder.finalize();
// Sparse array now represents an array like
// [None, Some("foo"), None, Some("bar"), None, None, None]
```

This array supports several queries including

- size() -> u64: returns the total number of possible spaces. i.e. the length of the bitvector
- num_elem() -> u64: returns the number of populated elements in the array
- num_elem_at(i: u64) -> u64 : Returns the number of populated elements in the array up to and **including** element i
- get_index_of(i: u64) -> Option<u64>: Returns the index where the ith present element occurs. Returns None if fewer than i elements are present
- get_at_index(i: u64) -> Option<&T>: Returns a reference to the element at position i if an element is present
- get_at_rank(i: u64) -> Returns a reference to the ith present element in the array if one exists. Otherwise returns None

```rust
sparse_array.size(); // 7
sparse_array.num_elem(); // 2
sparse_array.num_elem_at(3); // 2
sparse_array.get_at_rank(2); // Some(&String::from("bar"))
sparse_array.get_at_index(2); // None
sparse_array.get_at_index(3); // Some(&String::from("bar"))
sparse_array.get_index_of(2); // Option(3)
sparse_array.get_index_of(100); // None
```

The sparse array may also be constructed via the `from_dense` method which accepts an argument that implements an iterator of optional elements.

```rust
let sparse = SparseArray::from_dense(vec![Some(0), None, Some(1), None, None]);
assert_eq!(2, sparse.get_index_of(2).unwrap());
```

Additionally like the other structs sparse array supports an `overhead()` which returns the size in bits of the bitvector, rank_support, and select_support structures.

Finally it may be saved and loaded to and from a file

```rust
let sa = SparseArray::<u32>::from_dense_vec(vec![None, Some(1), None, Some(2), Some(3)]);
assert_eq!(*sa.get_at_index(1).unwrap(), 1);
assert_eq!(sa.get_index_of(1).unwrap(), 1);
assert_eq!(sa.num_elem_at(1), 1);
sa.save("tmp-file.bin").unwrap();
let loaded: SparseArray<u32> = SparseArray::load("tmp-file.bin").unwrap();
assert_eq!(*loaded.get_at_index(1).unwrap(), 1);
assert_eq!(loaded.get_index_of(1).unwrap(), 1);
assert_eq!(loaded.num_elem_at(1), 1);
```

## Citations

Kanda Shunsuke. 2018. Sucds. https://github.com/kampersanda/sucds

Tolnay David and Tryzelaar Erick. 2017. Serde. https://github.com/serde-rs/serde

2015. Clap-rs. https://github.com/clap-rs
