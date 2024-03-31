pub mod random_rollout;
pub mod mcts;
mod action;
mod termination;

use rand::Rng;
pub use random_rollout::random_rollout;
pub use mcts::Mcts;
pub use termination::Outcome;

pub trait Determinable<P, A, G: Mcts<P, A>, R: Rng + Sized> {
    fn determine(&self, rng: &mut R, perspective_player: P) -> G;
}
