use rand::{Rng, SeedableRng};

mod number_game;
mod perfect_info_game;


#[test]
fn tree_test() {
    let game = number_game::NumberGame::default();

    let mut rng = rand_pcg::Pcg32::seed_from_u64(0);
}
