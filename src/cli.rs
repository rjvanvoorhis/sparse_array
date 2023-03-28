use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(ValueEnum, Clone, Debug, Serialize)]
pub enum QueryMode {
    Rank,
    Select,
}

#[derive(Subcommand, Debug, Clone, Serialize)]
pub enum RankSelectCommands {
    Rank(RankArgs),
    Select(SelectArgs),
}

#[derive(ValueEnum, Debug, Clone, Serialize)]
pub enum BlockSize {
    Dynamic,
    Fixed,
}

#[derive(Parser, Debug, Serialize)]
pub struct RankSelectArgs {
    #[command(subcommand)]
    pub command: RankSelectCommands,

    pub outfile: PathBuf,
}

#[derive(Args, Debug, Clone, Serialize)]
pub struct RankArgs {
    #[arg(long, default_value = "1000")]
    /// The minimum length of the array to build
    pub min_size: u64,

    #[arg(long, default_value = "100000")]
    /// The maximum length of the array to build
    pub max_size: u64,

    #[arg(short, long, default_value = "1000")]
    pub step_size: usize,

    #[arg(short, long, default_value = "100")]
    pub query_size: u64,

    #[arg(short, long, default_value = "dynamic", value_enum)]
    pub block_size: BlockSize,
}

#[derive(Args, Debug, Clone, Serialize)]
pub struct SelectArgs {
    #[arg(long, default_value = "1000")]
    /// The minimum length of the array to build
    pub min_size: u64,

    #[arg(long, default_value = "100000")]
    /// The maximum length of the array to build
    pub max_size: u64,

    #[arg(short, long, default_value = "1000")]
    pub step_size: usize,

    #[arg(short, long, default_value = "100")]
    pub query_size: u64,

    #[arg(short, long, default_value = "dynamic", value_enum)]
    pub block_size: BlockSize,
}

#[derive(ValueEnum, Clone, Debug, Serialize)]
pub enum SparseQueryMode {
    NumElemAt,
    GetAtIndex,
    GetIndexOf,
}

#[derive(Parser, Debug, Serialize)]
pub struct RankSupportArgs {
    #[arg(value_enum)]
    /// Toggle between rank and select queries
    pub query_mode: QueryMode,
    pub outfile: PathBuf,

    #[arg(long, default_value = "1000")]
    /// The minimum length of the array to build
    pub min_size: u64,

    #[arg(long, default_value = "100000")]
    /// The maximum length of the array to build
    pub max_size: u64,

    #[arg(short, long, default_value = "1000")]
    pub step_size: usize,

    #[arg(short, long, default_value = "100")]
    pub query_size: u64,
}

#[derive(Parser, Debug, Clone, Serialize)]
pub struct SparseArrayArgs {
    #[command(subcommand)]
    pub command: SparseArrayCommands,

    pub outfile: PathBuf,
}

#[derive(Subcommand, Debug, Clone, Serialize)]
pub enum SparseArrayCommands {
    Sparsity(VarySparsityArgs),
    Length(VaryLengthArgs),
}

#[derive(Args, Debug, Clone, Serialize)]
pub struct VarySparsityArgs {
    #[arg(value_enum)]
    pub query_mode: SparseQueryMode,

    // pub outfile: PathBuf,
    #[arg(long, default_value = "100000")]
    pub length: u64,
    #[arg(long, default_value="0", value_parser = clap::value_parser!(u8).range(0..=100))]
    pub min_sparsity: u8,
    #[arg(long, default_value="5", value_parser = clap::value_parser!(u8).range(0..=100))]
    pub max_sparsity: u8,
    #[arg(short, long, default_value = "30")]
    pub step_size: usize,
    #[arg(short, long, default_value = "5")]
    pub query_size: u64,
}

#[derive(Args, Debug, Clone, Serialize)]
pub struct VaryLengthArgs {
    #[arg(value_enum)]
    pub query_mode: SparseQueryMode,

    // pub outfile: PathBuf,
    #[arg(long, default_value="15", value_parser = clap::value_parser!(u8).range(0..=100))]
    pub sparsity: u8,
    #[arg(long, default_value = "1000")]
    pub min_length: u64,
    #[arg(long, default_value = "100000")]
    pub max_length: u64,
    #[arg(short, long, default_value = "1000")]
    pub step_size: usize,
    #[arg(short, long, default_value = "100")]
    pub query_size: u64,
}
