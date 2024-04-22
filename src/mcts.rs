use crate::strategy::Strategy;
use crate::types::{Action, GameState};
use rand::rngs::ThreadRng;

pub struct StrategyMCTS {}
impl Strategy for StrategyMCTS {
    fn play(&mut self, state: &GameState, _: &mut ThreadRng) -> Action {
        Action::Put { mask: 1 }
    }
}
