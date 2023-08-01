use chess::{Board, CacheTable, ChessMove, Color};
use std::sync::mpsc::Receiver;
use std::time::SystemTime;

use crate::consts;
use crate::evaluation::evaluate_board;
use crate::search::find_best_move;
use crate::utils::CacheData;
use crate::utils::Eval;
use crate::utils::HashtableResultType;
use crate::utils::{EngineReturn, Statistics};

pub async fn enter_engine(
    board: Board,
    stdout_log: bool,
    stop_engine_rcv: Option<Receiver<bool>>,
) -> (ChessMove, Option<EngineReturn>) {
    if stdout_log {
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
    let mut abort_search: bool = false; // Flag invoked by the UCI layer to abort deepening

    let mut terminal_depth: i16 = 1; // Starting depth

    let mut best_score: Eval = Eval { score: 0 };
    let mut best_mve: ChessMove = Default::default();
    let mut best_line: [ChessMove; consts::DEPTH_LIM as usize] = Default::default();

    while (t_start.elapsed().unwrap() < consts::TIME_LIM)
        && (terminal_depth <= consts::DEPTH_LIM)
        && (!abort_search)
    {
        // Run until we hit the timelimit

        // If we get a stop command from the UCI layer, bail out of deepening
        match &stop_engine_rcv {
            Some(rcv) => abort_search = rcv.try_recv().unwrap_or(false),
            None => {},
        }

        if stdout_log {
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
            if best_mve != Default::default() {Some(best_mve)} else {None},
        );

        match search_result {
            Ok(result) => {
                (best_score, best_mve, best_line) = result;
                run_stats.depth_reached += 1;
            }
            Err(_) => {
                if stdout_log {
                    println!("Depth aborted")
                }
            }
        }

        // Go farther each iteration
        terminal_depth += 1;

        if stdout_log {
            println!(
                "Best move: {}, board score of best move (global): {}",
                best_mve, best_score.score
            );
        }
    }

    if stdout_log {
        println!(
            "Best move: {}, board score of best move: {}",
            best_mve, best_score.score
        );
    }

    if stdout_log {
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
    }

    let percent_reduction: f32 =
        (1.0 - (run_stats.searched_nodes as f32) / (run_stats.all_nodes as f32)) * 100.0;

    // get final time
    let end_time = SystemTime::now();
    if let Ok(duration) = end_time.duration_since(start_time) { run_stats.time_ms = duration.as_millis() as f32 }

    if consts::SEARCH_INFO && stdout_log {
        println!(
            "Search stats. \n All nodes in problem: {}\n Nodes visited {}, reduction {}%, times used cache {}, time elapsed (ms) {}",
            run_stats.all_nodes, run_stats.searched_nodes, percent_reduction, run_stats.caches_used, run_stats.time_ms,
        )
    }

    // Package up return data
    // TODO: make this cleaner so there is a single move return
    (
        best_mve,
        Some(EngineReturn {
            engine_move: best_mve.to_string(),
            engine_stats: Some(run_stats),
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::mpsc, sync::mpsc::Receiver, sync::mpsc::Sender};

    use super::*;
    use chess::{Board, Square};

    #[tokio::test]
    async fn test_integrated_engine() {
        let board: Board = Board::default(); // Initial board
        let (eng_move, _) = enter_engine(board, false, None).await;
        assert!(board.legal(eng_move)); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_stop_channel() {
        let board: Board = Board::default(); // Initial board
        let (_tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel(); // Stop channel
        let (eng_move, _) = enter_engine(board, false, Some(rx)).await;
        assert!(board.legal(eng_move)); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_board_post_engine() {
        let board: Board = Board::default(); // Initial board
        let board_orig = board.clone(); // Deep copy of board
        let _eng_move = enter_engine(board, false, None).await;
        assert_eq!(board, board_orig); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_queen_blunder() {
        // This sequence was a known queen blunder from a previous revision
        // run an integration test to make sure we don't make it again

        let board: Board =
            Board::from_str("r4rk1/pq3ppp/2p5/2PpP3/2pP4/P1P3R1/4QPPP/R5K1 b - - 0 1").unwrap();
        let (eng_move, _) = enter_engine(board, false, None).await;

        assert_ne!(eng_move, ChessMove::new(Square::E2, Square::B2, None))
    }
}
