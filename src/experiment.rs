use eyre::{Result, WrapErr};
use rand::{
    distributions::{Bernoulli, Uniform},
    rngs::StdRng,
    Rng,
};
use serde::Serialize;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use sucds::BitVector;

#[derive(Serialize)]
pub struct Iteration {
    p: u64,
    overhead: u64,
    query_time: Duration,
    setup_time: Duration,
}

#[derive(Serialize)]
pub struct Params {
    pub start: u64,
    pub stop: u64,
    pub step_size: u64,
    pub query_size: u64,
}

pub trait Experiment: Serialize {
    type Resource;

    fn setup(&self, rng: &mut StdRng, p: u64) -> Self::Resource;
    fn get_params(&self) -> &Params;
    fn get_overhead(&self, resource: &Self::Resource) -> u64;
    fn execute_query(&self, resource: &Self::Resource, p: u64) -> Duration;

    fn get_iterations(&self, rng: &mut StdRng) -> Vec<Iteration> {
        let params = self.get_params();
        (params.start..=params.stop)
            .step_by(params.step_size as usize)
            .map(|p| {
                let now = Instant::now();
                let resource = self.setup(rng, p);
                let setup_time = now.elapsed();
                let overhead = self.get_overhead(&resource);
                let query_dist = Uniform::new_inclusive(0, p);
                let mut query_time = Duration::default();
                rng.sample_iter(query_dist)
                    .take(params.query_size as usize)
                    .for_each(|x| {
                        query_time += self.execute_query(&resource, x);
                    });
                Iteration {
                    p,
                    overhead,
                    query_time,
                    setup_time,
                }
            })
            .collect()
    }

    fn save<S: AsRef<Path>>(&self, fname: S) -> Result<()> {
        let file = File::create(fname).wrap_err("could not create experiment output file")?;
        let mut writer = BufWriter::new(file);
        write!(
            &mut writer,
            "{}",
            serde_json::to_string(self).wrap_err("Could not serialize experiment")?
        )?;
        Ok(())
    }

    fn push_iteration(&mut self, iteration: Iteration);

    fn run(&mut self, rng: &mut StdRng) {
        self.get_iterations(rng)
            .into_iter()
            .for_each(|iteration| self.push_iteration(iteration))
    }
}

pub fn generate_bitvector_of_size(size: u64, rng: &mut StdRng) -> BitVector {
    let distribution = Bernoulli::new(0.5).unwrap();
    BitVector::from_bits(
        rng.sample_iter(&distribution)
            .take(size as usize)
            .collect::<Vec<bool>>(),
    )
}
