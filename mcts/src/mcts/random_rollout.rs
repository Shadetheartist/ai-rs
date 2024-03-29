use rand::Rng;
use crate::mcts::MCTS;
use crate::mcts::Termination;

pub fn random_rollout<
    'p,
    R: Rng + Sized,
    P,
    A,
    G: MCTS<'p, A, P> + Clone
>(game: &G, rng: &mut R) -> &'p P {
    let mut game = game.clone();

    loop {
        if let Some(termination) = game.terminal() {
            match termination {
                Termination::Winner(player) => return player,
                Termination::Escape => {}
            }
        }

        let mut actions = game.actions();

        let random_index = rng.gen_range(0..actions.len());

        let random_action = actions.remove(random_index);

        game = game.apply_action(random_action, rng).unwrap();
    }
}
