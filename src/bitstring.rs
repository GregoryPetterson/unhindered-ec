use std::borrow::Borrow;

use rand::{rngs::ThreadRng, Rng};

use crate::individual::Individual;
use crate::population::Population;

pub type Bitstring = Vec<bool>;

pub fn count_ones(bits: &[bool]) -> i64 {
    bits.iter().filter(|&&bit| bit).count() as i64
}

pub fn make_bitstring(len: usize, rng: &mut ThreadRng) -> Bitstring {
    (0..len).map(|_| rng.gen_bool(0.5)).collect()
}

impl Individual<Bitstring> {
    pub fn new_bitstring<R>(bit_length: usize, compute_fitness: impl Fn(&R) -> i64, rng: &mut ThreadRng) -> Individual<Bitstring> 
    where
        Bitstring: Borrow<R>,
        R: ?Sized
    {
        Individual::new(
                |rng| make_bitstring(bit_length, rng), 
                compute_fitness,
                rng)
    }
}

// Todo: Change this to use the new parameterized constructor.
impl Population<Bitstring> {
    pub fn new_bitstring(pop_size: usize, bit_length: usize) -> Population<Bitstring> {
        Population::new(
            pop_size,
            |rng| make_bitstring(bit_length, rng),
            count_ones
        )
    }
}
