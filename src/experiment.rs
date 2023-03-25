use std::{
    fs::File,
    io::{BufWriter, Write},
    iter::StepBy,
    path::Path,
    time::{Duration, Instant},
};

use eyre::{Context, Result};
use rand::rngs::StdRng;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ExperimentRun<P> {
    overhead: u64,
    parameter: P,
    setup_duration: Duration,
    query_duration: Duration,
}

#[derive(Serialize, Debug)]
pub struct ExperimentParams {
    pub query_size: u64,
    pub min_size: u64,
    pub max_size: u64,
    pub step_size: usize,
}

impl ExperimentParams {
    pub fn new(query_size: u64, min_size: u64, max_size: u64, step_size: usize) -> Self {
        Self {
            query_size,
            min_size,
            max_size,
            step_size,
        }
    }
}

impl<'a> IntoIterator for &'a ExperimentParams {
    type Item = u64;
    type IntoIter = StepBy<std::ops::RangeInclusive<u64>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.min_size..=self.max_size).step_by(self.step_size)
    }
}

pub trait Experiment: Serialize {
    type Resource;
    type Param;
    type I: Iterator<Item = Self::Param>;

    fn setup(&self, rng: &mut StdRng, param: &Self::Param) -> Self::Resource;
    fn get_overhead(&self, resource: &Self::Resource) -> u64;
    fn iter_params(&self) -> Self::I;
    fn execute_queries(&self, rng: &mut StdRng, resource: &Self::Resource) -> Duration;
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
    fn execute_run(&self, rng: &mut StdRng, parameter: Self::Param) -> ExperimentRun<Self::Param> {
        let now = Instant::now();
        let resource = self.setup(rng, &parameter);
        let setup_duration = now.elapsed();
        let overhead = self.get_overhead(&resource);
        let query_duration = self.execute_queries(rng, &resource);
        ExperimentRun {
            overhead,
            parameter,
            setup_duration,
            query_duration,
        }
    }

    fn create_runs(&self, rng: &mut StdRng) -> Vec<ExperimentRun<Self::Param>> {
        self.iter_params()
            .map(|p| self.execute_run(rng, p))
            .collect()
    }
}
