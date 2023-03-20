use std::time::Instant;

use clap::Parser;
use eyre::Result;
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;
use sparse_array::{
    args::SelectArguments,
    experiment::{generate_bitvector_of_size, Experiment, Iteration, Params},
    rank_support::RankSupport,
    select_support::SelectSupport,
};

#[derive(Serialize)]
pub struct SelectExperiment {
    runs: Vec<Iteration>,
    params: Params,
}

impl SelectExperiment {
    pub fn new(params: Params) -> Self {
        SelectExperiment {
            runs: Vec::new(),
            params,
        }
    }
}

impl Experiment for SelectExperiment {
    type Resource = SelectSupport;

    fn get_params(&self) -> &Params {
        &self.params
    }

    fn get_overhead(&self, resource: &Self::Resource) -> u64 {
        resource.overhead()
    }

    fn setup(&self, rng: &mut rand::rngs::StdRng, p: u64) -> Self::Resource {
        let store = generate_bitvector_of_size(p, rng);
        let rs = RankSupport::new_from_owned(store);
        SelectSupport::new_from_owned(rs)
    }

    fn execute_query(&self, resource: &Self::Resource, p: u64) -> std::time::Duration {
        let now = Instant::now();
        resource.select0(p);
        now.elapsed()
    }

    fn push_iteration(&mut self, iteration: Iteration) {
        self.runs.push(iteration);
    }
}

pub fn main() -> Result<()> {
    let args = SelectArguments::parse();
    let params = Params {
        query_size: args.query_size,
        start: args.min_value,
        stop: args.max_value,
        step_size: args.step_size,
    };
    let mut experiment = SelectExperiment::new(params);
    let mut rng = StdRng::seed_from_u64(42);
    experiment.run(&mut rng);
    experiment.save(args.outfile)?;
    Ok(())
}
