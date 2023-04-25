#![allow(clippy::missing_panics_doc)]

use std::{borrow::Borrow, iter::repeat_with};

use rand::rngs::ThreadRng;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{
    generator::Generator,
    individual::{self, Individual},
};

pub trait Population {
    type Individual;

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn size(&self) -> usize;
}

pub trait Generate: Population
where
    Self::Individual: Individual,
{
    fn generate<H>(
        pop_size: usize,
        make_genome: impl Fn(&mut ThreadRng) -> <Self::Individual as Individual>::Genome + Send + Sync,
        run_tests: impl Fn(&H) -> <Self::Individual as Individual>::TestResults + Send + Sync,
    ) -> Self
    where
        <Self::Individual as Individual>::Genome: Borrow<H>,
        H: ?Sized;
}

impl<I> Population for Vec<I> {
    type Individual = I;

    fn size(&self) -> usize {
        self.len()
    }
}

impl<I: individual::Generate + Send> Generate for Vec<I> {
    fn generate<H>(
        pop_size: usize,
        make_genome: impl Fn(&mut ThreadRng) -> <Self::Individual as Individual>::Genome + Send + Sync,
        run_tests: impl Fn(&H) -> <Self::Individual as Individual>::TestResults + Send + Sync,
    ) -> Self
    where
        <Self::Individual as Individual>::Genome: Borrow<H>,
        H: ?Sized,
    {
        (0..pop_size)
            .into_par_iter()
            .map_init(rand::thread_rng, |rng, _| {
                I::generate(&make_genome, &run_tests, rng)
            })
            .collect()
    }
}

pub struct GeneratorContext<IC> {
    population_size: usize,
    individual_context: IC,
}

impl<I, IC> Generator<Vec<I>, GeneratorContext<IC>> for ThreadRng
where
    Self: Generator<I, IC>,
    Vec<I>: Population,
{
    // We could implement this using Rayon's `par_bridge`, but we
    // have to replace `self.generate` with `thread_rng().generate`
    // since we can't pass `self` (a `ThreadRng`) around between
    // threads. Since we only generate populations rarely (typically
    // just once at the beginning) there's not a huge value in
    // parallelizing this action.
    fn generate(&mut self, context: &GeneratorContext<IC>) -> Vec<I> {
        repeat_with(|| self.generate(&context.individual_context))
            .take(context.population_size)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use rand::RngCore;

    use crate::individual::{ec::EcIndividual, Individual};

    use super::*;

    #[test]
    fn generate_works() {
        let vec_pop =
            Vec::<EcIndividual<_, _>>::generate(10, |rng| rng.next_u32() % 20, |g| (*g) + 100);
        assert!(vec_pop.is_empty().not());
        assert_eq!(10, vec_pop.size());
        #[allow(clippy::unwrap_used)] // The population shouldn't be empty
        let ind = vec_pop.into_iter().next().unwrap();
        assert!(*ind.genome() < 20);
        assert!(100 <= *ind.test_results() && *ind.test_results() < 120);
    }
}
