use std::fmt::Debug;
use rand::Rng;
use crate::mcts::Outcome;

pub trait Mcts<'p, P, A: Send>: Clone {
    type Error: Debug;

    fn actions(&self) -> &[A];
    fn apply_action<R: Rng + Sized>(&self, action: &A, rng: &mut R) -> Result<Self, Self::Error> where Self: Sized;
    fn outcome(&self) -> Option<Outcome<'p, P>>;

    fn current_player(&self) -> &'p P;
    fn players(&self) -> &[&'p P];
}