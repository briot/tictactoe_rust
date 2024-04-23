use crate::strategy::Strategy;
use crate::types::{Action, GameState};
use rand::rngs::ThreadRng;
use rand::Rng;

#[derive(Default)]
pub struct StrategyRandom {}

impl Strategy for StrategyRandom {
    fn name(&self) -> String {
        "Random".into()
    }

    fn play(&mut self, state: &GameState, rng: &mut ThreadRng) -> Action {
        let legal = state.legal_moves();
        let mut choice = rng.gen_range(0..legal.occupied.count_zeros());
        let mut current = 1;
        loop {
            if (legal.occupied & current) == 0 {
                if choice == 0 {
                    return Action::Put { mask: current };
                }
                choice -= 1;
            }
            current *= 2;
        }
    }
}
