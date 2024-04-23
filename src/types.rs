pub type Grid = u16;

#[derive(Debug)]
pub enum Action {
    Put { mask: Grid },
}

#[derive(Clone, Copy, Debug)]
pub enum Score {
    Player1Wins,
    Player2Wins,
    Draw,
    Undecided,
}

pub struct LegalMoves {
    pub occupied: Grid, //  bit is 1 if the cell is occupied
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct GameState {
    pub player1: Grid,    // bit set to 1 if player1 occupies the cell
    pub player2: Grid,    // bit set to 1 if player1 occupies the cell
    pub is_player1: bool, // true if next to play is player1
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
            Score::Undecided
        }
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn img(state: &GameState, bit: Grid) -> char {
            if state.player1 & bit != 0 {
                'X'
            } else if state.player2 & bit != 0 {
                'O'
            } else {
                '.'
            }
        }
        writeln!(f, "{:?} {} to play", self.score(),
           (if self.is_player1 { "player1" } else { "player2" })
        )?;
        writeln!(f, "{} {} {}", img(self, 1), img(self, 2), img(self, 4))?;
        writeln!(f, "{} {} {}", img(self, 8), img(self, 16), img(self, 32))?;
        writeln!(f, "{} {} {}", img(self, 64), img(self, 128), img(self, 256))?;
        Ok(())
    }
}
