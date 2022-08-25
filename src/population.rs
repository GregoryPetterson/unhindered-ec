#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::borrow::Borrow;

use rand::rngs::ThreadRng;
use rayon::prelude::{ParallelExtend, IntoParallelIterator, ParallelIterator};

use crate::individual::Individual;

pub struct Population<T> {
    pub individuals: Vec<Individual<T>>,
}

impl<T: Send> Population<T> {
    /*
     * See the lengthy comment in `individual.rs` on why we need the
     * whole `Borrow<R>` business.
     */
    pub fn new<R>(
            pop_size: usize,
            make_genome: impl Fn(&mut ThreadRng) -> T + Send + Sync, 
            compute_score: impl Fn(&R) -> i64 + Send + Sync) 
        -> Self
    where
        T: Borrow<R>,
        R: ?Sized
    {
        let mut individuals = Vec::with_capacity(pop_size);
        individuals.par_extend((0..pop_size)
            .into_par_iter()
            .map_init(
                rand::thread_rng,
                |rng, _| {
                    Individual::new(&make_genome, &compute_score, rng)
                })
        );
        // let mut rng = rand::thread_rng();
        // for _ in 0..pop_size {
        //     let ind = Individual::new(bit_length, &mut rng);
        //     pop.push(ind);
        // }
        Self {
            individuals,
        }
    }
}

impl<T> Population<T> {
    /// # Panics
    ///
    /// Will panic if the vector of individuals is empty.
    #[must_use]
    pub fn best_score(&self) -> &Individual<T> {
        assert!(!self.individuals.is_empty());
        #[allow(clippy::unwrap_used)]
        self.individuals.iter().max_by_key(
                |ind| ind.score
            ).unwrap()
    }
}