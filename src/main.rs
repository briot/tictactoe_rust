mod errors;
use rand::Rng;

type Player = i8;

enum Action {
    Put { player: Player, cell: usize },
}

#[derive(Debug)]
enum Score {
    Player1Wins,
    Player2Wins,
    Draw,
    Unknown,
}

struct LegalMoves {
    mask: u32, //  each bit is whether the corresponding cell is legal
    count: u8,
    player: Player,
}
impl LegalMoves {
    pub fn random(&self) -> Action {
        let mut rng = rand::thread_rng();
        let mut choice = rng.gen_range(0..self.count);
        let mut current = 1;
        let mut cell: usize = 0;
        loop {
            if (self.mask & current) != 0 {
                if choice == 0 {
                    return Action::Put {
                        player: self.player,
                        cell,
                    };
                }
                choice -= 1;
            }
            cell += 1;
            current *= 2;
        }
    }
}

#[derive(Default)]
struct GameState {
    board: [i8; 9],
}
impl GameState {
    pub fn perform(self, action: Action) -> Self {
        match action {
            Action::Put { player, cell } => {
                let mut b = self.board;
                b[cell] = player;
                GameState { board: b }
            }
        }
    }

    pub fn legal_moves(&self, player: Player) -> LegalMoves {
        let mut count = 0;
        let mut mask: u32 = 0;
        let mut current: u32 = 1;
        for cell in 0_usize..9 {
            if self.board[cell] == 0 {
                count += 1;
                mask |= current;
            }
            current *= 2;
        }
        LegalMoves {
            count,
            mask,
            player,
        }
    }

    pub fn score(&self) -> Score {
        let row1 = self.board[0] + self.board[1] + self.board[2];
        if row1 == 3 {
            return Score::Player1Wins;
        } else if row1 == -3 {
            return Score::Player2Wins;
        }

        let row2 = self.board[3] + self.board[4] + self.board[5];
        if row2 == 3 {
            return Score::Player1Wins;
        } else if row2 == -3 {
            return Score::Player2Wins;
        }

        let row3 = self.board[6] + self.board[7] + self.board[8];
        if row3 == 3 {
            return Score::Player1Wins;
        } else if row3 == -3 {
            return Score::Player2Wins;
        }

        let col1 = self.board[0] + self.board[3] + self.board[6];
        if col1 == 3 {
            return Score::Player1Wins;
        } else if col1 == -3 {
            return Score::Player2Wins;
        }

        let col2 = self.board[1] + self.board[4] + self.board[7];
        if col2 == 3 {
            return Score::Player1Wins;
        } else if col2 == -3 {
            return Score::Player2Wins;
        }

        let col3 = self.board[2] + self.board[5] + self.board[8];
        if col3 == 3 {
            return Score::Player1Wins;
        } else if col3 == -3 {
            return Score::Player2Wins;
        }

        let diag1 = self.board[0] + self.board[4] + self.board[8];
        if diag1 == 3 {
            return Score::Player1Wins;
        } else if diag1 == -3 {
            return Score::Player2Wins;
        }

        let diag2 = self.board[6] + self.board[4] + self.board[2];
        if diag2 == 3 {
            return Score::Player1Wins;
        } else if diag2 == -3 {
            return Score::Player2Wins;
        }

        for j in 0..9 {
            if self.board[j] == 0 {
                return Score::Unknown;
            }
        }

        Score::Draw
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "{:2} {:2} {:2}",
            self.board[0], self.board[1], self.board[2]
        )?;
        writeln!(
            f,
            "{:2} {:2} {:2}",
            self.board[3], self.board[4], self.board[5]
        )?;
        writeln!(
            f,
            "{:2} {:2} {:2}",
            self.board[6], self.board[7], self.board[8]
        )?;
        Ok(())
    }
}

fn main() -> Result<(), crate::errors::Error> {
    let mut play1wins = 0;
    let mut play2wins = 0;
    let mut draw = 0;
    let mut played = 0;
    loop {
        let mut state = GameState::default();
        let mut player: Player = 1;
        played += 1;

        loop {
            let action = state.legal_moves(player).random();
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

            player = -player;
        }

        if played >= 1_000_000 {
            break;
        }
    }
    println!(
        "Played {}, play1 {:.2}%, play2 {:.2}%, draw {:.2}%",
        played,
        play1wins as f32 / played as f32 * 100.,
        play2wins as f32 / played as f32 * 100.,
        draw as f32 / played as f32 * 100.
    );
    Ok(())
}
