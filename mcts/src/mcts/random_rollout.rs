use rand::Rng;
use crate::mcts::Mcts;
use crate::mcts::Outcome;

pub fn random_rollout<
    R: Rng + Sized,
    P,
    A: Clone,
    G: Mcts<P, A> + Clone
>(game: &G, rng: &mut R) -> Outcome<P> {
    let mut game = game.clone();

    loop {
        if let Some(outcome) = game.outcome() {
            return outcome;
        }

        let actions = &game.actions()[..];
        let random_action = rand::seq::SliceRandom::choose(actions, rng);

        if let Some(action) = random_action {
            game = game.apply_action((*action).clone(), rng).unwrap();
        } else {
            return Outcome::Escape("No actions available.".to_string());
        }

    }
}
