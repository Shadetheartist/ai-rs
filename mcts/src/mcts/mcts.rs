use std::fmt::Debug;
use rand::Rng;
use crate::mcts::Outcome;

pub trait Mcts<P, A>: Clone {
    type Error: Debug;

    fn actions(&self) -> Vec<A>;
    fn apply_action<R: Rng + Sized>(&self, action: A, rng: &mut R) -> Result<Self, Self::Error> where Self: Sized;
    fn outcome(&self) -> Option<Outcome<P>>;

    fn current_player(&self) -> P;
    fn players(&self) -> Vec<P>;
}