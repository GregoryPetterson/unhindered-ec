use rand::rngs::ThreadRng;

mod composable;

pub use composable::Composable;

pub trait Operator<Input>: Composable {
    type Output;

    fn apply(&self, input: Input, rng: &mut ThreadRng) -> Self::Output;
}
