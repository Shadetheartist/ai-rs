use rand::Rng;
use crate::mcts::Termination;

pub trait MCTS<'p, P, A> {
    type Player;
    type Action;
    type Error;
    fn current_player(&self) -> &'p P;
    fn actions(&self) -> &[A];
    fn apply_action<R: Rng + Sized>(&self, action: &A, rng: &mut R) -> Result<Self, Self::Error>;
    fn terminal(&self) -> Option<Termination<'p, P>>;
}