use crate::strategy::Strategy;
use crate::types::{Action, GameState, Score};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The game is represented as a tree of nodes, but the nodes are only created
/// as they are visited (so that if we had a large branching factor, we limit
/// memory usage).  The nodes do not store the board state.  Instead, it can
/// be computed by following from the root node to the child node and replaying
/// the actions.
///
/// In practice, tic-tac-toe only has 5477 valid states (out of 19683 different
/// positions), so it is advantageous to cache those states in a hash map, and
/// share the tree nodes.
/// In terms of MCTS, it means that a given node will in general have multiple
/// parents, and when we compute the "score" for a node, we should take into
/// account the total number of times any of the parents was visited.

struct Node {
    visited: u32,   // Number of times the node was visited
    wins: u32,      // Number of times this node lead to a winning game.
    moves: [Option<Rc<RefCell<Node>>>; 9],  // Each of the valid child nodes
}
const NOT_EXPLORED: Option<Rc<RefCell<Node>>> = None;

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
    tree: HashMap<GameState, Rc<RefCell<Node>>>,
}

impl StrategyMCTS {
    /// Returns 1 if full game was a win, 0 otherwise
    fn search_one(
        &mut self,
        state: &GameState,
        current_player_is_1: bool,
        rng: &mut ThreadRng,
    ) -> u32 {
        let node = self.tree[state].clone();
        let parent_visits = node.borrow().visited;

        // Choose one child
        // We implement the UCBT algorithm: for each child node, we compute
        //    ucb1 = wins/visited + c * sqrt( ln(parent_visits) / visited)
        // The first term encourages nodes that have had a better win ratio.
        // The second term encourages nodes that haven't been visited in a
        // while.
        // We select the child with the highest ucb1

        let legal = state.legal_moves();
        let mut best = (f32::NEG_INFINITY, 0);

        // Shuffle things, so that in case of equality we do not always use
        // the first choice available.
        let mut vec: Vec<usize> = (0..9).collect();
        vec.shuffle(rng);

        for idx in vec {
            if (legal.occupied & (1 << idx)) != 0 {
                // position is already occupied
                continue;
            }

            let ucb1 = match &node.borrow().moves[idx] {
                None => {
                    // never visited
                    f32::INFINITY
                }
                Some(child_node) => {
                    // We might arrive at the same position via different
                    // parents.  In this case, it is possible that the positions
                    // "visited" is non-zero, but the "parent_visits" is zero
                    let c = child_node.borrow();
                    if parent_visits == 0 {
                        //  ??? Should use the sum of the parent's visits
                        f32::INFINITY
                    } else {
                        c.wins as f32 / c.visited as f32
                            + 1.4
                                * ((parent_visits as f32).ln()
                                    / c.visited as f32)
                                    .sqrt()
                    }
                }
            };
            if ucb1 > best.0 {
                best = (ucb1, idx);
            }
        }

        // Explore that child
        let action = Action::Put { mask: 1 << best.1 };

        // ??? Should modify in place, for efficiency, if the board is large
        let next_state = state.perform(action);

        // Create new node if needed
        let child_node = {
            let mut n = node.borrow_mut();
            if n.moves[best.1].is_none() {
                let a = match self.tree.get(&next_state) {
                    None => {
                        let a: Rc<RefCell<Node>> = Rc::default();
                        self.tree.insert(next_state, a.clone());
                        a
                    }
                    Some(a) => {
                        assert!(
                            a.borrow().visited != 0,
                            "created earlier, but never marked as visited {}",
                            next_state,
                        );
                        a.clone()
                    }
                };
                n.moves[best.1] = Some(a.clone());
                a
            } else {
                self.tree[&next_state].clone()
            }
        };

        let result = match (current_player_is_1, next_state.score()) {
            (true, Score::Player1Wins) | (false, Score::Player2Wins) => 1,
            (_, Score::Player2Wins) | (_, Score::Player1Wins) => 0,
            (_, Score::Draw) => 1,
            (_, Score::Undecided) => {
                self.search_one(&next_state, current_player_is_1, rng)
            }
        };

        let mut c = child_node.borrow_mut();
        c.visited += 1;
        c.wins += result;

        result
    }
}

impl Strategy for StrategyMCTS {
    fn name(&self) -> String {
        "MonteCarlo".into()
    }

    fn play(&mut self, state: &GameState, rng: &mut ThreadRng) -> Action {
        //  The first  time, we basically do "offline" training and do a longer
        //  exploration.  Afterwards, we just do a few iterations to further
        //  improve the search.
        let iterations = if self.tree.is_empty() { 30000 } else { 100 };

        let node = match self.tree.get(state) {
            None => {
                let n: Rc<RefCell<Node>> = Rc::default();
                self.tree.insert(*state, n.clone());
                n
            }
            Some(node) => node.clone(),
        };

        // arbitrary limitations: 5477 is the total number of valide states in
        // the game, so if we have already visited very often, stop searching.
        if node.borrow().visited < 5477 {
            for _ in 0..iterations {
                let result = self.search_one(state, state.is_player1, rng);
                let mut n = node.borrow_mut();
                n.visited += 1;
                n.wins += result;
            }
        }

        // Now select the child with the highest win rate
        let mut best = (-1., 0);
        for idx in 0..9 {
            match &node.borrow().moves[idx] {
                None => {}
                Some(child_n) => {
                    let c = child_n.borrow();
                    let rate = c.wins as f32 / c.visited as f32;
                    if rate > best.0 {
                        best = (rate, idx);
                    }
                }
            }
        }

        Action::Put { mask: 1 << best.1 }
    }
}
