use std::{iter::StepBy, ops::RangeInclusive, path::PathBuf, time::Instant};

use eyre::Result;
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, SeedableRng};
use serde::Serialize;
use sparse_array::{
    cli::{SparseArrayArgs, SparseArrayCommands, SparseQueryMode},
    experiment::{Experiment, ExperimentRun},
    sparse_array::SparseArray,
};

use clap::Parser;

#[derive(Serialize)]
pub struct ExperimentContainer {
    pub runs: Vec<ExperimentRun<u64>>,
    pub args: SparseArrayArgs,
}

impl ExperimentContainer {
    pub fn new(args: SparseArrayArgs) -> Self {
        Self {
            args,
            runs: Vec::new(),
        }
    }

    pub fn get_query_mode(&self) -> &SparseQueryMode {
        match &self.args.command {
            SparseArrayCommands::Sparsity(values) => &values.query_mode,
            SparseArrayCommands::Length(values) => &values.query_mode,
        }
    }

    pub fn get_query_size(&self) -> u64 {
        match &self.args.command {
            SparseArrayCommands::Sparsity(values) => values.query_size,
            SparseArrayCommands::Length(values) => values.query_size,
        }
    }

    pub fn get_outfile(&self) -> &PathBuf {
        &self.args.outfile
    }

    pub fn run(&mut self, rng: &mut StdRng) {
        self.runs.extend(self.create_runs(rng));
    }
}

impl Experiment for ExperimentContainer {
    type Resource = SparseArray<String>;
    type Param = u64;
    type I = StepBy<RangeInclusive<u64>>;

    fn iter_params(&self) -> Self::I {
        match &self.args.command {
            SparseArrayCommands::Sparsity(value) => {
                (value.min_sparsity as u64..=value.max_sparsity as u64).step_by(value.step_size)
            }
            SparseArrayCommands::Length(value) => {
                (value.min_length..=value.max_length).step_by(value.step_size)
            }
        }
    }

    fn get_overhead(&self, resource: &Self::Resource) -> u64 {
        resource.overhead()
    }

    fn setup(&self, rng: &mut rand::rngs::StdRng, param: &Self::Param) -> Self::Resource {
        println!("Setting up run with parameter: {param}");
        let (sparsity, length) = match &self.args.command {
            SparseArrayCommands::Sparsity(value) => (*param as u8, value.length),
            SparseArrayCommands::Length(value) => (value.sparsity, *param),
        };
        let mut builder = SparseArray::create(length);
        let distribution = Uniform::<u8>::new(0, 100);
        distribution
            .sample_iter(rng)
            .take(length as usize)
            .enumerate()
            .filter(|&x| x.1 < sparsity)
            .enumerate()
            .for_each(|(sparse_idx, (idx, _))| {
                builder.append(format!("item_{sparse_idx}"), idx as u64)
            });
        builder.finalize()
    }

    fn execute_queries(
        &self,
        rng: &mut rand::rngs::StdRng,
        resource: &Self::Resource,
    ) -> std::time::Duration {
        let query_mode = self.get_query_mode();
        let query_size = self.get_query_size();
        let query_distribution = match query_mode {
            SparseQueryMode::NumElemAt | SparseQueryMode::GetAtIndex => {
                Uniform::new_inclusive(0, resource.size())
            }
            SparseQueryMode::GetIndexOf => Uniform::new_inclusive(0, resource.num_elem()),
            // QueryMode::Select => Uniform::new_inclusive(0, resource.size()),
            // QueryMode::Rank => Uniform::new_inclusive(0, resource.num_elem()),
        };
        query_distribution
            .sample_iter(rng)
            .take(query_size as usize)
            .map(|p| match query_mode {
                SparseQueryMode::NumElemAt => {
                    let now = Instant::now();
                    resource.num_elem_at(p);
                    now.elapsed()
                }
                SparseQueryMode::GetIndexOf => {
                    let now = Instant::now();
                    resource.get_index_of(p);
                    now.elapsed()
                }
                SparseQueryMode::GetAtIndex => {
                    let now = Instant::now();
                    resource.get_at_index(p);
                    now.elapsed()
                }
            })
            .sum()
    }
}

pub fn main() -> Result<()> {
    let args = SparseArrayArgs::parse();
    let mut experiment = ExperimentContainer::new(args);
    let mut rng = StdRng::seed_from_u64(42);
    experiment.run(&mut rng);
    experiment.save(experiment.get_outfile())?;
    Ok(())
}
