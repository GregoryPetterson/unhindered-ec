pub mod args;

use anyhow:: Result;
use clap::Parser;
use ec_core::{individual::scorer::FnScorer, test_results::{self, TestResults}, uniform_distribution_of};
use ordered_float::OrderedFloat;
use push::{genome::plushy::{ConvertToGeneGenerator, Plushy}, instruction::{FloatInstruction, PushInstruction}, push_vm::{program::PushProgram, push_state::PushState}};
use rand::{rngs::ThreadRng, thread_rng, Rng, RngCore};
use crate::args::{Args, RunModel};

fn training_inputs(num_cases: usize, rng: &mut ThreadRng) -> Vec<(i8, i8, i8)> {
    // Inputs from in the range [-100, 100] inclusive
    (0..num_cases)
    .map(|_| {
        (
            rng.gen_range(-100..=100),
            rng.gen_range(-100..=100),
            rng.gen_range(-100..=100),
        )
    })
    .collect()
}

fn median((x, y, z): (i8, i8, i8)) -> i8 {
    let mut sorted_values = [x, y, z];
    sorted_values.sort();
    return sorted_values[1];
}

fn training_cases(num_cases: usize, rng: &mut ThreadRng) -> Vec<((i8, i8, i8), i8)> {
    let inputs = training_inputs(num_cases, rng);
    inputs.into_iter().map(|input| (input, median(input))).collect()
}


// Build  push state needs to have multiple variables

// Check smallest for reference
// fn build_push_state(
//     program: impl DoubleEndedIterator<Item = PushProgram> + ExactSizeIterator,
//     input: i8
// ) -> PushState {
//     #[allow(clippy::unwrap_used)]
//     PushState::builder()
//     .with_max_stack_size(64)
//     .with_program(program)
//     .unwrap()
//     .with_float_input("x", input)
//     .build()

// }

fn main() -> Result <()> {
    let Args {
        run_model,
        population_size,
        max_initial_instructions,
        max_genome_length,
        num_generations
    } = Args::parse();

    let mut rng = thread_rng();

    let training_cases = training_cases(population_size, &mut rng);

    println!("Training cases: {training_cases:#?}");

    // Is the success of Median based on whatever is left on the integer stack?
    
    // let gene_generator = uniform_distribution_of![<PushInstruction>
    // FloatInstruction::Add,
    // ]
    // .into_gene_generator();

    for generation_number in 0..num_generations {
        match run_model {
            RunModel::Serial => generation.serial_next()?,
            RunModel::Parrallel => generation.par_next()?
        }
    }

    // We're defining a scorer function to be used to score a given generation
    let scorer = FnScorer(|genome: &Plushy| -> TestResults<test_results::Error<i8>> {
        // We need to clone our program so we can score it.
        let program = Vec::<PushProgram>::from(genome.clone());
        let errors: TestResults<test_results::Error<i8>> = training_cases
            .iter()
            .map(|&((a, b, c), expected)| {
                #[allow(clippy::unwrap_used)]
                let state = PushState::builder()
                .with_max_stack_size(max_initial_instructions)
                .with_program(program.clone())
                
                
            })
            .collect();
    errors
    });

    Ok(())
}

// What remains to be done? Need to be able to score the 
