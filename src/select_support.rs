use std::rc::Rc;

use eyre::Result;

use crate::{binary_search::bisect_left, rank_support::RankSupport};

#[derive(Debug)]
pub struct SelectSupport {
    pub rank_support: Rc<RankSupport>,
}

impl SelectSupport {
    pub fn new(rank_support: Rc<RankSupport>) -> Self {
        Self { rank_support }
    }

    pub fn new_from_owned(rank_support: RankSupport) -> Self {
        Self {
            rank_support: Rc::new(rank_support),
        }
    }

    pub fn select1(&self, value: u64) -> u64 {
        bisect_left(0, self.rank_support.store.len() as u64, |x| {
            self.rank_support.rank1(x).cmp(&value)
        })
    }

    pub fn select0(&self, value: u64) -> u64 {
        bisect_left(0, self.rank_support.store.len() as u64, |x| {
            self.rank_support.rank0(x).cmp(&value)
        })
    }

    pub fn save(&self, fname: &str) -> Result<()> {
        self.rank_support.save(fname)?;
        Ok(())
    }

    pub fn load(&self, fname: &str) -> Result<Self> {
        let rank_support = RankSupport::load(fname)?;
        Ok(Self::new_from_owned(rank_support))
    }
}
