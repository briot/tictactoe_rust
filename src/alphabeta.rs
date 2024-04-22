use crate::strategy::Strategy;
use crate::types::{Action, GameState, Grid, Score};
use rand::rngs::ThreadRng;
use std::collections::HashMap;

#[derive(Default)]
pub struct StrategyAlphaBeta {
    cache: HashMap<GameState, (Score, Grid)>,
}
impl StrategyAlphaBeta {
    fn play_with_score(&mut self, state: &GameState) -> (Score, Grid) {
        // ??? Could go faster by checking symmetries and rotations
        if let Some(b) = self.cache.get(state) {
            return *b;
        }

        let legal = state.legal_moves();
        let mut best_score: Score = Score::Undecided;
        let mut best_play_mask: Grid = 0;
        for current in 0..=8 {
            let mask = 1 << current;
            if (legal.occupied & mask) == 0 {
                let next_state = state.perform(Action::Put { mask });
                let mut score = next_state.score();
                if let Score::Undecided = score {
                    let (s, _) = self.play_with_score(&next_state);
                    score = s;
                }

                match (state.is_player1, score, best_score) {
                    (true, Score::Player1Wins, _) => {
                        return (Score::Player1Wins, mask);
                    }
                    (false, Score::Player1Wins, Score::Undecided) => {
                        best_score = Score::Player1Wins;
                        best_play_mask = mask;
                    }
                    (true, Score::Player2Wins, Score::Undecided) => {
                        best_score = Score::Player2Wins;
                        best_play_mask = mask;
                    }
                    (false, Score::Player2Wins, _) => {
                        return (Score::Player2Wins, mask);
                    }
                    (true, Score::Draw, Score::Player2Wins)
                    | (false, Score::Draw, Score::Player1Wins)
                    | (_, Score::Draw, Score::Undecided) => {
                        // draw is better than letting the other player win
                        best_score = Score::Draw;
                        best_play_mask = mask;
                    }
                    (false, Score::Player1Wins, _)
                    | (true, Score::Player2Wins, _)
                    | (_, Score::Draw, _) => {
                        // We already have a better strategy, ignore this one
                    }
                    (_, Score::Undecided, _) => {
                        panic!("should not happen");
                    }
                }
            }
        }
        self.cache.insert(*state, (best_score, best_play_mask));
        (best_score, best_play_mask)
    }
}

impl Strategy for StrategyAlphaBeta {
    fn play(&mut self, state: &GameState, _: &mut ThreadRng) -> Action {
        let (_, m) = self.play_with_score(state);
        Action::Put { mask: m }
    }
}
