mod errors;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;
use std::thread::available_parallelism;

#[derive(Debug)]
enum Action {
    Put { mask: u16 },
}

#[derive(Clone, Copy, Debug)]
enum Score {
    Player1Wins,
    Player2Wins,
    Draw,
    Unknown,
}

struct LegalMoves {
    occupied: u16, //  each bit is whether the corresponding cell is legal
}

trait Strategy {
    fn play(&mut self, state: &GameState, rng: &mut ThreadRng) -> Action;
}

struct StrategyRandom {}
impl Strategy for StrategyRandom {
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

#[derive(Default)]
struct StrategyAlphaBeta {
    cache: HashMap<GameState, (Score, u16)>,
}
impl StrategyAlphaBeta {
    fn play_with_score(&mut self, state: &GameState) -> (Score, u16) {
        // ??? Could go faster by checking symmetries and rotations
        if let Some(b) = self.cache.get(state) {
            return *b;
        }

        let legal = state.legal_moves();
        let mut best_score: Score = Score::Unknown;
        let mut best_play_mask: u16 = 0;
        for current in 0..=8 {
            let mask = 1 << current;
            if (legal.occupied & mask) == 0 {
                let next_state = state.perform(Action::Put { mask });
                let mut score = next_state.score();
                if let Score::Unknown = score {
                    let (s, _) = self.play_with_score(&next_state);
                    score = s;
                }

                match (state.is_player1, score, best_score) {
                    (true, Score::Player1Wins, _) => {
                        return (Score::Player1Wins, mask);
                    }
                    (false, Score::Player1Wins, Score::Unknown) => {
                        best_score = Score::Player1Wins;
                        best_play_mask = mask;
                    }
                    (true, Score::Player2Wins, Score::Unknown) => {
                        best_score = Score::Player2Wins;
                        best_play_mask = mask;
                    }
                    (false, Score::Player2Wins, _) => {
                        return (Score::Player2Wins, mask);
                    }
                    (true, Score::Draw, Score::Player2Wins)
                    | (false, Score::Draw, Score::Player1Wins)
                    | (_, Score::Draw, Score::Unknown) => {
                        // draw is better than letting the other player win
                        best_score = Score::Draw;
                        best_play_mask = mask;
                    }
                    (false, Score::Player1Wins, _)
                    | (true, Score::Player2Wins, _)
                    | (_, Score::Draw, _) => {
                        // We already have a better strategy, ignore this one
                    }
                    (_, Score::Unknown, _) => {
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

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct GameState {
    player1: u16,     // bit set to 1 if player1 occupies the cell
    player2: u16,     // bit set to 1 if player1 occupies the cell
    is_player1: bool, // true if next to play is player1
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            player1: !0b111111111, // so that count_zeros only looks at board
            player2: !0b111111111,
            is_player1: true,
        }
    }
}

impl GameState {
    pub fn perform(&self, action: Action) -> Self {
        match action {
            Action::Put { mask } => {
                if self.is_player1 {
                    GameState {
                        player1: self.player1 | mask,
                        player2: self.player2,
                        is_player1: false,
                    }
                } else {
                    GameState {
                        player1: self.player1,
                        player2: self.player2 | mask,
                        is_player1: true,
                    }
                }
            }
        }
    }

    pub fn legal_moves(&self) -> LegalMoves {
        LegalMoves {
            occupied: self.player1 | self.player2,
        }
    }

    pub fn score(&self) -> Score {
        if self.player1 & 0b000000111 == 0b000000111
            || self.player1 & 0b000111000 == 0b000111000
            || self.player1 & 0b111000000 == 0b111000000
            || self.player1 & 0b100100100 == 0b100100100
            || self.player1 & 0b010010010 == 0b010010010
            || self.player1 & 0b001001001 == 0b001001001
            || self.player1 & 0b100010001 == 0b100010001
            || self.player1 & 0b001010100 == 0b001010100
        {
            Score::Player1Wins
        } else if self.player2 & 0b000000111 == 0b000000111
            || self.player2 & 0b000111000 == 0b000111000
            || self.player2 & 0b111000000 == 0b111000000
            || self.player2 & 0b100100100 == 0b100100100
            || self.player2 & 0b010010010 == 0b010010010
            || self.player2 & 0b001001001 == 0b001001001
            || self.player2 & 0b100010001 == 0b100010001
            || self.player2 & 0b001010100 == 0b001010100
        {
            Score::Player2Wins
        } else if (self.player1 | self.player2) == !0 {
            Score::Draw
        } else {
            Score::Unknown
        }
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn img(state: &GameState, bit: u16) -> char {
            if state.player1 & bit != 0 {
                'X'
            } else if state.player2 & bit != 0 {
                'O'
            } else {
                '.'
            }
        }
        writeln!(f, "{:?}", self.score())?;
        writeln!(f, "{} {} {}", img(self, 1), img(self, 2), img(self, 4))?;
        writeln!(f, "{} {} {}", img(self, 8), img(self, 16), img(self, 32))?;
        writeln!(f, "{} {} {}", img(self, 64), img(self, 128), img(self, 256))?;
        Ok(())
    }
}

fn play<Strategy1: Strategy, Strategy2: Strategy>(
    max_count: u32,
    player1: &mut Strategy1,
    player2: &mut Strategy2,
) -> (u32, u32, u32, u32) {
    let mut play1wins = 0;
    let mut play2wins = 0;
    let mut draw = 0;
    let mut played = 0;
    let mut rng = rand::thread_rng();

    loop {
        let mut state = GameState::default();
        played += 1;

        loop {
            let action = if state.is_player1 {
                player1.play(&state, &mut rng)
            } else {
                player2.play(&state, &mut rng)
            };

            state = state.perform(action);
            match state.score() {
                Score::Player1Wins => {
                    play1wins += 1;
                    break;
                }
                Score::Player2Wins => {
                    play2wins += 1;
                    break;
                }
                Score::Draw => {
                    draw += 1;
                    break;
                }
                Score::Unknown => {}
            }
        }

        if played >= max_count {
            break;
        }
    }
    (played, play1wins, play2wins, draw)
}

fn main() -> Result<(), crate::errors::Error> {
    const TOTAL_RUNS: u32 = 1_000_000;
    let cores: u32 = available_parallelism().unwrap().get().try_into().unwrap();

    // https://math.stackexchange.com/questions/4045893/if-two-computers-are-playing-tic-tac-toe-but-they-are-choosing-their-squares-ra
    // Random vs Random:
    //    theory says 58.49% of wins for player1, 28.81% for player1,
    //    and 12.70% draw.
    // Random vs Perfect:
    //    if first player is perfect:
    //       wins with 191/192 = 99.48% of wins
    //       draws with 1/192 = 0.52%
    //    if second player is perfect:
    //       wins with 887/945 = 93.86%
    //       draws with 43/945 =  4.55%
    //       loses with 1/945  =  1.06%

    let handles = (0..cores)
        .map(|_| {
            std::thread::spawn(move || {
                let mut player1 = StrategyRandom {};
                let mut player2 = StrategyAlphaBeta::default();
                play(TOTAL_RUNS / cores, &mut player1, &mut player2)
            })
        })
        .collect::<Vec<_>>();

    let mut played = 0;
    let mut p1 = 0;
    let mut p2 = 0;
    let mut draw = 0;
    for h in handles {
        let (tp, tp1, tp2, td) = h.join().unwrap();
        played += tp;
        p1 += tp1;
        p2 += tp2;
        draw += td;
    }

    println!(
        "total {} play1 {:.2}%, play2 {:.2}%, draw {:.2}%",
        played,
        p1 as f32 / played as f32 * 100.,
        p2 as f32 / played as f32 * 100.,
        draw as f32 / played as f32 * 100.
    );
    Ok(())
}
