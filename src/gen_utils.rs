use std::ops::{Generator, GeneratorState};

pub fn gen_to_iter<A, G: Generator<Return = (), Yield = A>>(gen: G) -> impl Iterator<Item = A> {
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

impl<G: Generator<Return = ()>> Iterator for GeneratorIter<G> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            GeneratorIterState::Empty => None,
            GeneratorIterState::Pending => {
                match unsafe { self.gen.resume() } {
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
