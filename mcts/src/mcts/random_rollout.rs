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

        let mut actions = game.actions();

        let random_index = rng.gen_range(0..actions.len());

        let random_action = actions.remove(random_index);

        game = game.apply_action(random_action, rng).unwrap();
    }
}
