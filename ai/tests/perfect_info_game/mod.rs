use std::ops::Range;
use rand::{Rng};
use mcts::{Mcts, Outcome};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct PogPlayer {
    name: String,
    alive: bool,
    numbers: Vec<u8>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Pog {
    players: Vec<PogPlayer>,
    state: PogState
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum PogState {
    None
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum PogAction {
    None
}


impl Default for Pog {
    fn default() -> Self {
        Pog { players: vec![], state: PogState::None }
    }
}

impl Mcts<usize, PogAction> for Pog {
    type Error = ();

    fn actions(&self) -> Vec<PogAction> {
        let mut actions = Vec::new();

        actions
    }

    fn apply_action<R: Rng + Sized>(&self, action: PogAction, rng: &mut R) -> Result<Self, Self::Error> where Self: Sized {
        let game = self.clone();

        Ok(game)
    }

    fn outcome(&self) -> Option<Outcome<usize>> {
        todo!()
    }

    fn current_player(&self) -> usize {
       todo!()
    }

    fn players(&self) -> Vec<usize> {
        (0..self.players.len()).collect()
    }
}
