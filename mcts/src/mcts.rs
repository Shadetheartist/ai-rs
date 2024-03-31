pub mod random_rollout;
pub mod mcts;
mod action;
mod termination;

use rand::Rng;
pub use random_rollout::random_rollout;
pub use mcts::Mcts;
pub use termination::Outcome;

pub trait Determinable<'p, 'r, P, A: Send, G: Mcts<'p, P, A>, R: Rng + Sized> {
    fn determine(&self, rng: &mut R, perspective_player: &'p P) -> G;
}

// extension trait design pattern
impl <'p, 'r, P, A: Send, G: Mcts<'p, P, A>, R: Rng + Sized> Determinable<'p, 'r, P, A, G, R> for G {
    fn determine(&self, _rng: &mut R, _perspective_player: &'p P) -> G {
        todo!()
    }
}
