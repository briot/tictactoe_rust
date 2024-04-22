mod alphabeta;
mod errors;
mod mcts;
mod random;
mod strategy;
mod types;
use crate::alphabeta::StrategyAlphaBeta;
use crate::mcts::StrategyMCTS;
use crate::random::StrategyRandom;
use crate::strategy::Strategy;
use crate::types::{GameState, Score};
use std::thread::available_parallelism;

///
/// Strategies for playing the game
///

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
                Score::Undecided => {}
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
