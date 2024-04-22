mod errors;
use rand::Rng;

type Player = i8;

#[derive(Debug)]
enum Action {
    Put { player: Player, mask: u32 },
}

#[derive(Debug)]
enum Score {
    Player1Wins,
    Player2Wins,
    Draw,
    Unknown,
}

struct LegalMoves {
    occupied: u32, //  each bit is whether the corresponding cell is legal
    player: Player,
}
impl LegalMoves {
    pub fn random(&self) -> Action {
        let mut rng = rand::thread_rng();
        let mut choice = rng.gen_range(0..self.occupied.count_zeros());
        let mut current = 1;
        loop {
            if (self.occupied & current) == 0 {
                if choice == 0 {
                    return Action::Put {
                        player: self.player,
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
    player1: u32,   // bit set to 1 if player1 occupies the cell
    player2: u32,   // bit set to 1 if player1 occupies the cell
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            player1: !0b111111111,  // so that count_zeros only looks at board
            player2: !0b111111111,
        }
    }
}

impl GameState {
    pub fn perform(self, action: Action) -> Self {
        match action {
            Action::Put { player, mask } => {
                let mut next = GameState {
                    player1: self.player1,
                    player2: self.player2,
                };
                if player == 1 {
                    next.player1 |= mask;
                } else {
                    next.player2 |= mask;
                }
                next
            }
        }
    }

    pub fn legal_moves(&self, player: Player) -> LegalMoves {
        LegalMoves {
            occupied: self.player1 | self.player2,
            player,
        }
    }

    pub fn score(&self) -> Score {
        if   self.player1 & 0b000000111 == 0b000000111
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
        fn img(state: &GameState, bit: u32) -> char {
            if state.player1 & bit != 0 { 'X' }
            else if state.player2 & bit != 0 { 'O' }
            else { '.' }
        }
        writeln!(f, "{:?}", self.score())?;
        writeln!(f, "{} {} {}", img(self, 1), img(self, 2), img(self, 4))?;
        writeln!(f, "{} {} {}", img(self, 8), img(self, 16), img(self, 32))?;
        writeln!(f, "{} {} {}", img(self, 64), img(self, 128), img(self, 256))?;
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
