use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use rand::{Rng, RngCore};
use crate::mcts::{Outcome};
use crate::mcts::Mcts;
use crate::mcts::random_rollout;

pub trait Determinable<P, A, G: Mcts<P, A>> {
    fn determine<R: Rng>(&self, rng: &mut R, perspective_player: P) -> G;
}

type Determinizations<A, P> = Vec<HashMap<A, HashMap<P, f64>>>;


#[allow(dead_code)]
pub fn ismcts_mt<
    R: Rng + RngCore + Sized + Clone + Send,
    P: Eq + PartialEq + Hash + Send + Sync,
    A: Eq + PartialEq + Hash + Send + Sync + Clone,
    G: Mcts<P, A> + Determinable<P, A, G> + Send
>(game: &G, rng: &R, num_determinizations: usize, num_simulations: usize) -> A {

    // actions should be the same between all determinizations
    // so, we can pre-calculate the actions, then just copy them into each thread
    let actions = game.actions();

    let determinization_scores: Arc<Mutex<Determinizations<A, P>>> = Arc::new(Mutex::new(Vec::new()));

    thread::scope(|scope| {
        for determinization_idx in 0..num_determinizations {
            {
                let actions = actions.clone();

                let mut rng = clone_and_advance_rng(rng, determinization_idx);

                let determinization_scores = determinization_scores.clone();

                let current_player = game.current_player();

                let game = game.determine(&mut rng, current_player);

                scope.spawn(move || {
                    let mut action_scores: HashMap<A, HashMap<P, f64>> = HashMap::new();

                    for action in actions.iter() {
                        let game_after_action = game.apply_action(action.clone(), &mut rng).unwrap();

                        let mut scores: HashMap<P, f64> = HashMap::new();
                        for _simulation_count in 0..num_simulations {
                            let outcome = random_rollout(&game_after_action, &mut rng);

                            match outcome {
                                Outcome::Winner(winner) => {
                                    *scores.entry(winner).or_insert(0f64) += 1f64;
                                }
                                Outcome::Winners(_) => unimplemented!(),
                                Outcome::Escape(_) => {}
                            }
                        }

                        // normalize score values, might be a mistake
                        let max = scores.values().fold(0f64, |sum, &val| if sum > val { sum } else { val });
                        scores.iter_mut().for_each(|(_, v)| *v /= max);

                        action_scores.insert(action.clone(), scores);
                    }

                    determinization_scores.lock().unwrap().push(action_scores);
                });
            }
        }
    });

    game.actions()[0].clone()
}

fn clone_and_advance_rng<R: Rng + RngCore + Sized + Clone + Send>(rng: &R, delta: usize) -> R {
    // clone & shadow the rng so each thread has its own copy
    let mut rng = rng.clone();

    // advance the RNG by jumping ahead 'determinization_idx' number of jumps before
    // applying a determinization, that way each determinization is unique
    for _ in 0..delta {
        rng.next_u32();
    }

    rng
}

#[derive(Clone)]
pub struct ISMCTSPlayerParams {
    pub num_determinations: usize,
    pub num_simulations_per_action: usize,
}

pub struct ISMCTSParams {
    pub seed: u64,
    pub num_sims: usize,
    pub max_cores: usize,
    pub sim_players: Vec<ISMCTSPlayerParams>,
}

impl Default for ISMCTSParams {
    fn default() -> Self {
        Self {
            seed: 0,
            num_sims: 1,
            max_cores: 24,
            sim_players: vec![
                ISMCTSPlayerParams {
                    num_determinations: 12,
                    num_simulations_per_action: 100,
                },
                ISMCTSPlayerParams {
                    num_determinations: 12,
                    num_simulations_per_action: 100,
                },
                ISMCTSPlayerParams {
                    num_determinations: 12,
                    num_simulations_per_action: 100,
                },
            ],
        }
    }
}
