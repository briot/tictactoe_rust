use crate::strategy::Strategy;
use crate::types::{Action, GameState, Score};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

struct Node {
    moves: [Option<Box<Node>>; 9],
    visited: u32,
    wins: u32,
}
const NOT_EXPLORED: Option<Box<Node>> = None;

impl Default for Node {
    fn default() -> Self {
        Node {
            moves: [NOT_EXPLORED; 9],
            visited: 0,
            wins: 0,
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(wins={}, visited={})", self.wins, self.visited)
    }
}

#[derive(Default)]
pub struct StrategyMCTS {
    tree: Node,
}

impl StrategyMCTS {
    fn search_one(
        node: &mut Node,
        state: &GameState,
        current_player_is_1: bool,
        parent_visits: u32,
        rng: &mut ThreadRng,
    ) -> u32 {  // returns 1 if full game was a win, 0 otherwise

        // Choose one child
        // We implement the UCBT algorithm: for each child node, we compute
        //    ucb1 = wins/visited + c * sqrt( ln(parent_visits) / visited)
        // The first term encourages nodes that have had a better win ratio.
        // The second term encourages nodes that haven't been visited in a
        // while.
        // We select the child with the highest ucb1

        let mut legal = state.legal_moves();
        let mut best = (-1000., 0);

        // Shuffle things, so that in case of equality we do not always use
        // the first choice available.
        let mut vec: Vec<usize> = (0..9).collect();
        vec.shuffle(rng);

        for idx in vec {
            if (legal.occupied & 1) != 0 {
                // position is already occupied
                legal.occupied >>= 1;
                continue;
            }

            let ucb1 = match &node.moves[idx] {
                None => {
                    // never visited
                    f32::INFINITY
                }
                Some(node) => {
                    node.wins as f32 / node.visited as f32
                    + 1.4 * (
                        (parent_visits as f32).ln() / node.visited as f32
                    ).sqrt()
                }
            };
            if ucb1 > best.0 {
                best = (ucb1, idx);
            }

            legal.occupied >>= 1;
        }

        // Create new node if needed
        if node.moves[best.1].is_none() {
            node.moves[best.1] = Some(Box::default());
        }

        // Explore that child
        let action = Action::Put { mask: 1 << best.1};

        // ??? Should modify in place, for efficiency
        let next_state = state.perform(action);
        let result: u32;
        match (current_player_is_1, next_state.score()) {
            (true, Score::Player1Wins) 
            | (false, Score::Player2Wins) => {
                result = 1;
                node.moves[best.1].as_mut().unwrap().wins += 1;
                node.moves[best.1].as_mut().unwrap().visited += 1;
            }
            (_, Score::Player2Wins)
            | (_, Score::Player1Wins) => {
                result = 0;
                node.moves[best.1].as_mut().unwrap().visited += 1;
            }
            (_, Score::Draw) => {
                result = 1;
                node.moves[best.1].as_mut().unwrap().wins += 1;
                node.moves[best.1].as_mut().unwrap().visited += 1;
            }
            (_, Score::Undecided) => {
                result = StrategyMCTS::search_one(
                   node.moves[best.1].as_mut().unwrap(),
                    &next_state,
                    current_player_is_1,
                    node.visited,
                    rng,
                );

                // Propagate result from child to current node
                node.visited += 1;
                node.wins += result;
            }
        }

        result

    }
}

impl Strategy for StrategyMCTS {
    fn name(&self) -> String {
        "MonteCarlo".into()
    }

    fn play(&mut self, state: &GameState, rng: &mut ThreadRng) -> Action {
        // Since the initial state has changed, we must reset the tree.
        self.tree = Node::default();

        for count in 0 .. 200 {
            StrategyMCTS::search_one(
                &mut self.tree,
                state,
                state.is_player1,
                count,
                rng,
            );
        }

        // Now select the child with the highest win rate
        let mut best = (-1., 0);
        for idx in 0 .. 9 {
            match &self.tree.moves[idx] {
                None => {},
                Some(n) => {
                    let rate = n.wins as f32 / n.visited as f32;
                    if rate > best.0 {
                        best = (rate, idx);
                    }
                }
            }
        }

        Action::Put { mask: 1 << best.1 }
    }
}
