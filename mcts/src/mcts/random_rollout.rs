use rand::Rng;
use crate::mcts::MCTS;
use crate::mcts::Outcome;

pub fn random_rollout<
    'p,
    R: Rng + Sized,
    P,
    A: Send,
    G: MCTS<'p, P, A> + Clone
>(game: &G, rng: &mut R) -> Outcome<'p, P> {
    let mut game = game.clone();

    loop {
        if let Some(outcome) = game.outcome() {
            return outcome;
        }

        let actions = game.actions();
        let random_action = rand::seq::SliceRandom::choose(actions, rng);

        if let Some(action) = random_action {
            game = game.apply_action(action, rng).unwrap();
        } else {
            return Outcome::Escape("No actions available.".to_string());
        }

    }
}
