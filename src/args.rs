use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
// Generates data for part 1 of write-up: bitvector rank
pub struct RankArguments {
    #[arg(default_value_t = 10, short, long)]
    pub min_value: u64,

    #[arg(default_value_t = 1000000, long)]
    pub max_value: u64,

    #[arg(default_value_t = 100000, long)]
    pub step_size: u64,

    /// the number of rank queries to execute per iteration
    #[arg(default_value_t = 10, short, long)]
    pub query_size: u64,

    /// a file to write the results to
    pub outfile: PathBuf,
}

#[derive(Debug, Parser)]
// Generates data for part 2 of write-up: bitvector rank
pub struct SelectArguments {
    #[arg(default_value_t = 10, short, long)]
    pub min_value: u64,

    #[arg(default_value_t = 1000000, long)]
    pub max_value: u64,

    #[arg(default_value_t = 100000, long)]
    pub step_size: u64,

    /// the number of rank queries to execute per iteration
    #[arg(default_value_t = 10, short, long)]
    pub query_size: u64,

    /// a file to write the results to
    pub outfile: PathBuf,
}
