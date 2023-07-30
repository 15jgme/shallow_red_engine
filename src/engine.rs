use chess::{Board, CacheTable, ChessMove, Color};
use std::ops::Add;
use std::ops::AddAssign;
use std::time::SystemTime;

use crate::consts;
use crate::evaluation::evaluate_board;
use crate::search::find_best_move;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Statistics {
    pub all_nodes: i32,
    pub searched_nodes: i32,
    pub caches_used: i32,
    pub time_ms: f32,
    pub depth_reached: u8,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EngineReturn {
    pub engine_move: String,
    pub engine_stats: Option<Statistics>,
}

pub(crate) fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b {
        a
    } else {
        b
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct CacheData {
    pub(crate) move_depth: i16,
    pub(crate) search_depth: i16,
    pub(crate) evaluation: Eval,
    pub(crate) move_type: HashtableResultType, // What type of move we have
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum HashtableResultType {
    RegularMove,
    PVMove,
    CutoffMove,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub(crate) struct Eval {
    pub(crate) score: i16, // Score is always (ALWAYS!) expressed as + is winning for White
}

impl Eval {
    pub(crate) fn for_colour(&self, colour: Color) -> i16 {
        // Returns the score for a colour where positive is more desriable for that colour
        match colour {
            Color::White => self.score,
            Color::Black => -self.score,
        }
    }
}

impl Add for Eval {
    type Output = Eval;

    fn add(self, rhs: Eval) -> Self::Output {
        Eval {
            score: self.score + rhs.score,
        }
    }
}

impl AddAssign for Eval {
    fn add_assign(&mut self, rhs: Eval) {
        self.score += rhs.score
    }
}

pub(crate) fn abs_eval_from_color(eval_rel: i16, color: Color) -> Eval {
    // Function provides a global eval struct from a local evaluation
    // specific to one colour, and the colour it is specific to.

    let eval_glob = match color {
        Color::White => eval_rel,  // + white
        Color::Black => -eval_rel, // Must be flipped for black
    };
    Eval { score: eval_glob }
}

pub(crate) fn flip_colour(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

pub async fn enter_engine(board: Board, verbose: Option<bool>) -> (ChessMove, Option<EngineReturn>) {
    let should_print = match verbose {
        Some(v) => match v {
            true => true,
            false => false,
        },
        None => false,
    };

    if should_print {
        println!("=============================================");
        println!("Balance of board {}", evaluate_board(board).score);
    }

    let start_time = SystemTime::now();

    let color_i: Color = board.side_to_move();

    // The color expressed as an integer, where white == 1 and black == -1

    let mut run_stats = Statistics {
        all_nodes: 0,
        searched_nodes: 0,
        caches_used: 0,
        time_ms: 0.0,
        depth_reached: 1,
    };

    // Declare cache table for transpositions
    let mut cache: CacheTable<CacheData> = CacheTable::new(
        67108864,
        CacheData {
            move_depth: 0,
            search_depth: 0,
            evaluation: Eval { score: 0 },
            move_type: HashtableResultType::RegularMove,
        },
    );

    let t_start = SystemTime::now(); // Initial time before running

    let mut terminal_depth: i16 = 1; // Starting depth

    let mut best_score: Eval = Eval { score: 0 };
    let mut best_mve: ChessMove = Default::default();
    let mut best_line: [ChessMove; consts::DEPTH_LIM as usize] = Default::default();

    while t_start.elapsed().unwrap() < consts::TIME_LIM && terminal_depth <= consts::DEPTH_LIM {
        // Run until we hit the timelimit
        if should_print {
            println!("Current depth {}", terminal_depth);
        }

        let search_result = find_best_move(
            board.clone(),
            0,
            terminal_depth,
            i16::min_value() + 1,
            i16::max_value() - 1,
            color_i,
            &mut run_stats,
            &mut cache,
            &t_start,
        );

        match search_result {
            Ok(result) => {
                (best_score, best_mve, best_line) = result;
                run_stats.depth_reached += 1;
            }
            Err(_) => {if should_print {println!("Depth aborted")}},
        }

        // Go farther each iteration
        terminal_depth += 1;

        if should_print {
            println!(
                "Best move: {}, board score of best move (global): {}",
                best_mve, best_score.score
            );
        }

    }

    if should_print {
        println!(
            "Best move: {}, board score of best move: {}",
            best_mve, best_score.score
        );
    }

    println!("Proposed line:");
    let mut i: i8 = 1;
    let mut is_white = color_i == Color::White;
    for mve in best_line {
        if is_white {
            println!("White, Move {}: {}", i, mve);
        } else {
            println!("Black, Move {}: {}", i, mve);
        }

        is_white = !is_white;
        i += 1;
    }

    let percent_reduction: f32 =
        (1.0 - (run_stats.searched_nodes as f32) / (run_stats.all_nodes as f32)) * 100.0;

    // get final time
    let end_time = SystemTime::now();
    match end_time.duration_since(start_time) {
        Ok(duration) => run_stats.time_ms = duration.as_millis() as f32,
        Err(_) => {}
    }

    if consts::SEARCH_INFO && should_print{
        println!(
            "Search stats. \n All nodes in problem: {}\n Nodes visited {}, reduction {}%, times used cache {}, time elapsed (ms) {}",
            run_stats.all_nodes, run_stats.searched_nodes, percent_reduction, run_stats.caches_used, run_stats.time_ms,
        )
    }

    // Package up return data
    // TODO: make this cleaner so there is a single move return
    return (
        best_mve,
        Some(EngineReturn {
            engine_move: best_mve.to_string(),
            engine_stats: Some(run_stats),
        }),
    );
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use chess::{Board, Square};

    #[tokio::test]
    async fn test_integrated_engine() {
        let board: Board = Board::default(); // Initial board
        let (eng_move, _) = enter_engine(board, Some(false)).await;
        assert!(board.legal(eng_move)); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_board_post_engine() {
        let board: Board = Board::default(); // Initial board
        let board_orig = board.clone(); // Deep copy of board
        let _eng_move = enter_engine(board, Some(false)).await;
        assert_eq!(board, board_orig); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_queen_blunder() {
        // This sequence was a known queen blunder from a previous revision
        // run an integration test to make sure we don't make it again

        let board: Board =
            Board::from_str("r4rk1/pq3ppp/2p5/2PpP3/2pP4/P1P3R1/4QPPP/R5K1 b - - 0 1").unwrap();

        let (eng_move, _) = enter_engine(board, Some(false)).await;

        assert_ne!(eng_move, ChessMove::new(Square::E2, Square::B2, None))
    }
}
