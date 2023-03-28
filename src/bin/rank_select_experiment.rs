use std::{iter::StepBy, mem, path::PathBuf, rc::Rc, time::Instant};

use clap::Parser;
use eyre::{Context, Result};
use rand::{
    distributions::{Bernoulli, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use serde::Serialize;
use sparse_array::{
    cli::{BlockSize, RankArgs, RankSelectArgs, RankSelectCommands, SelectArgs},
    experiment::{Experiment, ExperimentRun},
    rank_support::RankSupport,
    select_support::SelectSupport,
};
use sucds::BitVector;

pub fn generate_bitvector_of_size(size: u64, rng: &mut StdRng) -> BitVector {
    let distribution = Bernoulli::new(0.5).unwrap();
    BitVector::from_bits(
        rng.sample_iter(&distribution)
            .take(size as usize)
            .collect::<Vec<bool>>(),
    )
}

const WORD_SIZE: usize = mem::size_of::<usize>();

#[derive(Serialize)]
pub struct RankExperiment {
    params: RankArgs,
    runs: Vec<ExperimentRun<u64>>,
    outfile: PathBuf,
}

impl RankExperiment {
    pub fn new(params: RankArgs, outfile: PathBuf) -> Self {
        Self {
            runs: Vec::new(),
            params,
            outfile,
        }
    }
}

#[derive(Serialize)]
pub struct SelectExperiment {
    params: SelectArgs,
    runs: Vec<ExperimentRun<u64>>,
    outfile: PathBuf,
}

impl SelectExperiment {
    pub fn new(params: SelectArgs, outfile: PathBuf) -> Self {
        Self {
            runs: Vec::new(),
            params,
            outfile,
        }
    }
}

impl Experiment for RankExperiment {
    type Resource = RankSupport;
    type Param = u64;
    type I = StepBy<std::ops::RangeInclusive<u64>>;

    fn iter_params(&self) -> Self::I {
        (self.params.min_size..=self.params.max_size).step_by(self.params.step_size)
    }

    fn setup(&self, rng: &mut rand::rngs::StdRng, param: &Self::Param) -> Self::Resource {
        let store = generate_bitvector_of_size(*param, rng);
        match self.params.block_size {
            BlockSize::Dynamic => RankSupport::new_from_owned(store),
            BlockSize::Fixed => RankSupport::with_block_size(WORD_SIZE as u64, Rc::new(store)),
        }
    }

    fn get_overhead(&self, resource: &Self::Resource) -> u64 {
        resource.overhead()
    }

    fn execute_queries(&self, rng: &mut StdRng, resource: &Self::Resource) -> std::time::Duration {
        let query_distribution = Uniform::new_inclusive(0, resource.store.len() as u64);
        rng.sample_iter(query_distribution)
            .take(self.params.query_size as usize)
            .map(|x: u64| {
                let now = Instant::now();
                resource.rank1(x);
                now.elapsed()
            })
            .sum()
    }
}

impl Experiment for SelectExperiment {
    type Resource = SelectSupport;
    type Param = u64;
    type I = StepBy<std::ops::RangeInclusive<u64>>;

    fn iter_params(&self) -> Self::I {
        (self.params.min_size..=self.params.max_size).step_by(self.params.step_size)
    }

    fn setup(&self, rng: &mut rand::rngs::StdRng, param: &Self::Param) -> Self::Resource {
        let store = generate_bitvector_of_size(*param, rng);
        let rank_support = RankSupport::new_from_owned(store);
        SelectSupport::new_from_owned(rank_support)
    }

    fn get_overhead(&self, resource: &Self::Resource) -> u64 {
        resource.overhead()
    }

    fn execute_queries(&self, rng: &mut StdRng, resource: &Self::Resource) -> std::time::Duration {
        let query_distribution =
            Uniform::new_inclusive(0, resource.rank_support.store.len() as u64);
        rng.sample_iter(query_distribution)
            .take(self.params.query_size as usize)
            .map(|x: u64| {
                let now = Instant::now();
                resource.select1(x);
                now.elapsed()
            })
            .sum()
    }
}

pub enum RankSupportExperiment {
    Rank(RankExperiment),
    Select(SelectExperiment),
}

impl RankSupportExperiment {
    pub fn new(args: RankSelectArgs) -> Self {
        match args.command {
            RankSelectCommands::Rank(rank_args) => {
                Self::Rank(RankExperiment::new(rank_args, args.outfile))
            }
            RankSelectCommands::Select(select_args) => {
                Self::Select(SelectExperiment::new(select_args, args.outfile))
            }
        }
    }

    pub fn run(&mut self, rng: &mut StdRng) {
        match self {
            Self::Rank(experiment) => {
                experiment.runs.extend(experiment.create_runs(rng));
            }
            Self::Select(experiment) => {
                experiment.runs.extend(experiment.create_runs(rng));
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        match self {
            Self::Rank(experiment) => {
                experiment
                    .save(&experiment.outfile)
                    .wrap_err("Failed to save rank experiment results")?;
            }
            Self::Select(experiment) => {
                experiment
                    .save(&experiment.outfile)
                    .wrap_err("Failed to save select experiment results")?;
            }
        }
        Ok(())
    }
}

pub fn main() -> Result<()> {
    let args = RankSelectArgs::parse();
    let mut experiment = RankSupportExperiment::new(args);
    // let mut rng = StdRng::seed_from_u64(42);
    let mut rng = StdRng::from_entropy();
    experiment.run(&mut rng);
    experiment.save()?;
    Ok(())
}
