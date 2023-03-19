use std::{
    fs::File,
    io::BufWriter,
    io::Write,
    rc::Rc,
    time::{Duration, Instant},
};

use clap::Parser;
use eyre::{Context, Result};
use rand::{
    distributions::{Bernoulli, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use serde::{Deserialize, Serialize};
use sparse_array::{args::RankArguments, rank_support::RankSupport};
use sucds::BitVector;

#[derive(Serialize, Deserialize, Debug)]
pub struct Iteration {
    n: u64,
    overhead: u64,
    time: Duration,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Experiment {
    results: Vec<Iteration>,
    query_size: u64,
}

impl Experiment {
    pub fn new(query_size: u64) -> Self {
        Self {
            results: Vec::new(),
            query_size,
        }
    }

    pub fn add_iteration(&mut self, n: u64, rng: &mut StdRng) {
        let distribution = Bernoulli::new(0.5).unwrap();
        let query_range = Uniform::new_inclusive(0, n);
        let bv = BitVector::from_bits(
            rng.sample_iter(&distribution)
                .take(n as usize)
                .collect::<Vec<bool>>(),
        );
        // let rs = RankSupport::with_block_size(32, Rc::new(bv));
        let rs = RankSupport::new(Rc::new(bv));
        let mut counter: Duration = Duration::default();
        rng.sample_iter(&query_range)
            .take(self.query_size as usize)
            .for_each(|x| {
                let now = Instant::now();
                rs.rank1(x);
                counter += Instant::now() - now;
            });
        let iteration = Iteration {
            n,
            overhead: rs.overhead(),
            time: counter,
        };
        self.results.push(iteration);
        // Uniform::new_inclusive(0, n).sample_iter(rng).take(self.query_size as usize).for_each(f)
    }
}

fn main() -> Result<()> {
    let args = RankArguments::parse();
    let mut experiment = Experiment::new(args.query_size);
    let mut rng = StdRng::seed_from_u64(42);
    for n in (args.min_value..=args.max_value).step_by(args.step_size as usize) {
        // println!("Running experiment with n={n}");
        experiment.add_iteration(n, &mut rng);
    }
    let file = File::create(args.outfile).wrap_err("Failed to create file")?;
    let mut writer = BufWriter::new(file);
    write!(
        &mut writer,
        "{}",
        serde_json::to_string(&experiment).wrap_err("Could not serialize experiment")?
    )?;
    Ok(())
}
