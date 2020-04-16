use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

pub fn gen_to_iter<A, G>(gen: G) -> impl Iterator<Item=A>
    where G: Generator<Return=(), Yield=A> + Unpin {
    GeneratorIter {
        state: GeneratorIterState::Pending,
        gen,
    }
}

#[derive(PartialEq, Eq)]
pub struct GeneratorIter<G> {
    state: GeneratorIterState,
    gen: G,
}

#[derive(PartialEq, Eq)]
enum GeneratorIterState {
    Pending,
    Empty,
}

impl<G> Iterator for GeneratorIter<G> where G: Generator<Return=()> + Unpin {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            GeneratorIterState::Empty => None,
            GeneratorIterState::Pending => {
                match Pin::new(&mut self.gen).resume(()) {
                    GeneratorState::Yielded(value) => Some(value),
                    GeneratorState::Complete(_) => {
                        self.state = GeneratorIterState::Empty;
                        None
                    }
                }
            }
        }
    }
}
