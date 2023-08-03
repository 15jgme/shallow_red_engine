use chess::{Board, CacheTable, ChessMove, Color};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::SystemTime;

use crate::consts;
use crate::evaluation::evaluate_board;
use crate::managers::cache_manager::{Cache, CacheData, CacheInputGrouping, HashtableResultType};
use crate::managers::stats_manager::{Statistics, StatisticsDepth};
use crate::search::find_best_move;
use crate::utils::common::EngineReturn;
use crate::utils::common::Eval;
use crate::utils::engine_interface::EngineSettings;
use crate::utils::search_interface::SearchParameters;

pub async fn enter_engine(
    board: Board,
    settings: EngineSettings,
) -> (ChessMove, Option<EngineReturn>) {
    if settings.verbose {
        println!("=============================================");
        println!("Balance of board {}", evaluate_board(board).score);
    }

    let start_time = SystemTime::now();

    let color_i: Color = board.side_to_move();

    // The color expressed as an integer, where white == 1 and black == -1

    let mut depth_stats = StatisticsDepth::default();
    let mut search_stats: Statistics = Statistics::default();

    let (cache, cache_arc, cache_arc_thread, cache_tx);
    // Create the cache thread
    match settings.cache_settings {
        Some(cache_external) => {
            cache = cache_external.clone(); // CacheInputGroup is all cloneable stuff
        }
        None => {
            cache_arc = Arc::new(RwLock::new(Cache::default()));
            cache_arc_thread = cache_arc.clone();
            let (cache_tx_init, cache_rx) = Cache::generate_channel();
            cache_tx = cache_tx_init.clone();
            cache = CacheInputGrouping {
                cache_ref: cache_arc,
                cache_tx,
            };
            let _cache_thread_hndl = thread::spawn(move || {
                Cache::cache_manager_server(cache_arc_thread.clone(), cache_rx)
            });
        }
    }

    // Cache thread has been created

    let t_start = SystemTime::now(); // Initial time before running
    let mut abort_search: bool = false; // Flag invoked by the UCI layer to abort deepening

    let mut terminal_depth: i16 = 1; // Starting depth

    let mut best_score: Eval = Eval { score: 0 };
    let mut best_mve: ChessMove = Default::default();
    let mut best_line: [ChessMove; consts::DEPTH_LIM as usize] = Default::default();

    while (t_start.elapsed().unwrap() < settings.time_limit)
        && (terminal_depth <= consts::DEPTH_LIM)
        && (!abort_search)
    {
        // Run until we hit the timelimit

        // If we get a stop command from the UCI layer, bail out of deepening
        match &settings.stop_engine_rcv {
            Some(rcv) => abort_search = rcv.try_recv().unwrap_or(false),
            None => {}
        }

        if settings.verbose {
            println!("Current depth {}", terminal_depth);
        }

        let search_result = find_best_move(
            board.clone(),
            SearchParameters {
                depth: 0,
                depth_lim: terminal_depth,
                alpha: i16::min_value() + 1,
                beta: i16::max_value() - 1,
                color: color_i,
                cache: cache.clone(),
                t_start: &t_start,
                t_lim: settings.time_limit,
                first_search_move: if best_mve != Default::default() {
                    Some(best_mve)
                } else {
                    None
                },
            },
        );

        match search_result {
            Ok(result) => {
                let search_output = result;
                best_score = search_output.node_eval;
                best_mve = search_output.best_move;
                best_line = search_output.best_line;
                depth_stats.depth_reached += 1;
                search_stats = search_output.node_stats;
            }
            Err(_) => {
                if settings.verbose {
                    println!("Depth aborted")
                }
            }
        }

        // Go farther each iteration
        terminal_depth += 1;

        if settings.verbose {
            println!(
                "Best move: {}, board score of best move (global): {}",
                best_mve, best_score.score
            );
        }
    }

    if settings.verbose {
        println!(
            "Best move: {}, board score of best move: {}",
            best_mve, best_score.score
        );
    }

    if settings.verbose {
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
        (1.0 - (search_stats.searched_nodes as f32) / (search_stats.all_nodes as f32)) * 100.0;

    // get final time
    let end_time = SystemTime::now();
    if let Ok(duration) = end_time.duration_since(start_time) {
        depth_stats.time_ms = duration.as_millis() as f32
    }

    if consts::SEARCH_INFO && settings.verbose {
        println!(
            "Search stats. \n All nodes in problem: {}\n Nodes visited {}, reduction {}%, times used cache {}, time elapsed (ms) {}",
            search_stats.all_nodes, search_stats.searched_nodes, percent_reduction, search_stats.caches_used, depth_stats.time_ms,
        )
    }

    // Package up return data
    // TODO: make this cleaner so there is a single move return
    (
        best_mve,
        Some(EngineReturn {
            engine_move: best_mve.to_string(),
            engine_search_stats: Some(search_stats),
            engine_depth_stats: Some(depth_stats),
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
        let (eng_move, _) = enter_engine(board, EngineSettings::default()).await;
        assert!(board.legal(eng_move)); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_stop_channel() {
        let board: Board = Board::default(); // Initial board
        let (_tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel(); // Stop channel
        let (eng_move, _) = enter_engine(board, EngineSettings::default()).await;
        assert!(board.legal(eng_move)); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_board_post_engine() {
        let board: Board = Board::default(); // Initial board
        let board_orig = board.clone(); // Deep copy of board
        let _eng_move = enter_engine(board, EngineSettings::default()).await;
        assert_eq!(board, board_orig); // Make sure the engine move is legal
    }

    #[tokio::test]
    async fn test_queen_blunder() {
        // This sequence was a known queen blunder from a previous revision
        // run an integration test to make sure we don't make it again

        let board: Board =
            Board::from_str("r4rk1/pq3ppp/2p5/2PpP3/2pP4/P1P3R1/4QPPP/R5K1 b - - 0 1").unwrap();
        let (eng_move, _) = enter_engine(board, EngineSettings::default()).await;

        assert_ne!(eng_move, ChessMove::new(Square::E2, Square::B2, None))
    }
}
