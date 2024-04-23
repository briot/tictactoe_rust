use crate::types::{Action, GameState};
use rand::rngs::ThreadRng;

pub trait Strategy {
    fn name(&self) -> String;
    fn play(&mut self, state: &GameState, rng: &mut ThreadRng) -> Action;
}
