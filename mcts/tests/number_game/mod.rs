use std::ops::Range;
use std::path::Iter;
use rand::{Rng};
use mcts::{Mcts, Outcome};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NumberGamePlayer {
    name: String,
    alive: bool,
    numbers: Vec<u8>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NumberGame {
    guess_range: Range<u8>,
    players: Vec<NumberGamePlayer>,
    state: NumberGameState
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum NumberGameState {
    PlayerMustSelectNumber(usize, usize), //(player_idx, number_idx)
    PlayerMustGuessNumber(usize), //(player_idx)
    PlayerMustRespondToGuess(usize, u8), //(player_idx, opponent_player_idx, guess_number)
    Winner(usize)
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum NumberGameAction {
    SelectNumber(usize, usize, u8), // (player_idx, number_idx, guess_number)
    Guess(usize, usize, u8), // (player_idx, opponent_player_idx, guess_number)
    Respond(usize, Response), // (player_idx, response)
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum Response {
    GuessIsCorrect,
    GuessIsGreaterThanAction,
    GuessIsLesserThanActual,
    NoResponse,
}

impl Default for NumberGame {
    fn default() -> Self {
        NumberGame {
            guess_range: 1..20,
            players: vec![
                NumberGamePlayer { name: "Jim".to_string(), alive: true, numbers: vec![0, 0] },
                NumberGamePlayer { name: "Pam".to_string(), alive: true, numbers: vec![0, 0] },
                NumberGamePlayer { name: "Michael".to_string(), alive: true, numbers: vec![0, 0] },
            ],
            state: NumberGameState::PlayerMustSelectNumber(0, 0),
        }
    }
}


impl NumberGame {
    fn living_opponents(&self, player_idx: usize) -> impl Iterator<Item = usize> + '_ {
        self.players.iter().enumerate().filter(move |(idx, player)| *idx != player_idx && player.alive).map(|(idx, _)| idx)
    }

    fn naive_next_player_idx(&self, player_idx: usize) -> usize {
        player_idx + 1 % self.players.len()
    }

    fn next_living_player_idx(&self, player_idx: usize) -> usize {
        let mut naive_next_player_idx = self.naive_next_player_idx(player_idx);

        while self.players[naive_next_player_idx].alive == false {
            naive_next_player_idx = self.naive_next_player_idx(player_idx);
        }

        naive_next_player_idx
    }
}

impl Mcts<usize, NumberGameAction> for NumberGame {
    type Error = ();

    fn actions(&self) -> Vec<NumberGameAction> {
        let mut actions = Vec::new();

        match self.state {
            NumberGameState::PlayerMustSelectNumber(player_idx, number_idx) => {
                for n in self.guess_range.clone() {
                    actions.push(NumberGameAction::SelectNumber(player_idx, number_idx, n))
                }
            }
            NumberGameState::PlayerMustGuessNumber(player_idx) => {
                for opp_idx in self.living_opponents(player_idx) {
                    for n in self.guess_range.clone() {
                        actions.push(NumberGameAction::Guess(player_idx, opp_idx, n))
                    }
                }
            }
            NumberGameState::PlayerMustRespondToGuess(_, _) => {}
            NumberGameState::Winner(_) => {}
        }

        actions
    }


    fn apply_action<R: Rng + Sized>(&self, action: NumberGameAction, rng: &mut R) -> Result<Self, Self::Error> where Self: Sized {
        let mut game = self.clone();
        match action {
            NumberGameAction::SelectNumber(player_idx, number_idx, n) => {
                game.players[player_idx].numbers[number_idx] = n;
                if game.players[player_idx].numbers.contains(&0) {
                    game.state = NumberGameState::PlayerMustSelectNumber(player_idx, number_idx + 1);
                } else {
                    let next_player_idx = game.next_living_player_idx(player_idx);
                    if game.players[next_player_idx].numbers.contains(&0) {
                        // go to next player, who needs to select their numbers
                        game.state = NumberGameState::PlayerMustSelectNumber(next_player_idx, 0);
                    } else {
                        // all numbers are selected
                        // player 0 starts guessing numbers
                        game.state = NumberGameState::PlayerMustGuessNumber(0);
                    }
                }

                Ok(game)
            }
            NumberGameAction::Guess(player_idx, opponent_idx, n) => {

                Ok(game)
            }
            NumberGameAction::Respond(_, _) => {
                Ok(game)
            }
        }
    }

    fn outcome(&self) -> Option<Outcome<usize>> {
        match self.state {
            NumberGameState::Winner(player_idx) => Some(Outcome::Winner(player_idx)),
            _ => None,
        }
    }

    fn current_player(&self) -> usize {
        match self.state {
            NumberGameState::PlayerMustSelectNumber(player_idx, _) |
            NumberGameState::PlayerMustGuessNumber(player_idx) |
            NumberGameState::PlayerMustRespondToGuess(player_idx, _) |
            NumberGameState::Winner(player_idx) => player_idx,
        }
    }

    fn players(&self) -> Vec<usize> {
        (0..self.players.len()).collect()
    }
}
