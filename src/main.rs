mod errors;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::thread::available_parallelism;

#[derive(Debug)]
enum Action {
    Put { is_player1: bool, mask: u16 },
}

#[derive(Debug)]
enum Score {
    Player1Wins,
    Player2Wins,
    Draw,
    Unknown,
}

struct LegalMoves {
    occupied: u16, //  each bit is whether the corresponding cell is legal
    is_player1: bool,
}

trait Strategy {
    fn play(&mut self, legal: &LegalMoves, rng: &mut ThreadRng) -> Action;
}

struct StrategyRandom {}
impl Strategy for StrategyRandom {
    fn play(&mut self, legal: &LegalMoves, rng: &mut ThreadRng) -> Action {
        let mut choice = rng.gen_range(0..legal.occupied.count_zeros());
        let mut current = 1;
        loop {
            if (legal.occupied & current) == 0 {
                if choice == 0 {
                    return Action::Put {
                        is_player1: legal.is_player1,
                        mask: current,
                    };
                }
                choice -= 1;
            }
            current *= 2;
        }
    }
}

struct GameState {
    player1: u16, // bit set to 1 if player1 occupies the cell
    player2: u16, // bit set to 1 if player1 occupies the cell
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            player1: !0b111111111, // so that count_zeros only looks at board
            player2: !0b111111111,
        }
    }
}

impl GameState {
    pub fn perform(self, action: Action) -> Self {
        match action {
            Action::Put { is_player1, mask } => {
                let mut next = GameState {
                    player1: self.player1,
                    player2: self.player2,
                };
                if is_player1 {
                    next.player1 |= mask;
                } else {
                    next.player2 |= mask;
                }
                next
            }
        }
    }

    pub fn legal_moves(&self, is_player1: bool) -> LegalMoves {
        LegalMoves {
            occupied: self.player1 | self.player2,
            is_player1,
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
        let mut is_player1 = true;
        played += 1;

        loop {
            let legal = state.legal_moves(is_player1);
            let action = if is_player1 {
                player1.play(&legal, &mut rng)
            } else {
                player2.play(&legal, &mut rng)
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

            is_player1 = !is_player1;
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

    // Random vs Random:
    //    theory says 58.49% of wins for player1, 28.81% for player1,
    //    and 12.70% draw.

    let handles = (0..cores)
        .map(|_| std::thread::spawn(move || {
            let mut player1 = StrategyRandom {};
            let mut player2 = StrategyRandom {};
            play(TOTAL_RUNS / cores, &mut player1, &mut player2)
        }))
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
