use crate::consts::{self, USE_CACHE};

use crate::extensions::should_extend;
use crate::managers::cache_manager::{BoundType, CacheData, CacheEntry};
use crate::managers::stats_manager::Statistics;
// use crate::ordering::RetreivedCacheData;
use crate::quiescent::quiescent_search;
use crate::utils::common::min;
use crate::utils::search_interface::{SearchOutput, SearchParameters};
use crate::{
    ordering,
    utils::common::{flip_colour, max},
};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};

pub fn find_best_move<EvalFunc>(board: Board, mut params: SearchParameters<EvalFunc>) -> Result<SearchOutput, ()> where
EvalFunc: Fn(
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
) -> i16 + Clone,{
    let alpha_orig = params.alpha;

    let mut cache_pv_move: Option<ChessMove> = None;
    let mut cache_cutoff_move: Option<ChessMove> = None;

    // Internal stats data for this node and children
    let mut node_stats = Statistics::default();
    node_stats.searched_nodes += 1; // Get a point to the all nodes stats just by visiting
    let mut max_move = ChessMove::default();

    // ===================== Check TT for this node ===================== //
    let mut _skip_tt_push = false; // Flag for whether or not we should avoid pushing to the TT
                                  // Check if this move is in our cache (with a flag to disable cache lookup)
    match USE_CACHE && params.depth > 0 {
        true => {
            if let Some(cache_result) = params
                .cache
                .cache_ref
                .try_read_for(crate::consts::TT_MAXTIME_LOOKUP)
            {
                if let Some(cache_result) = cache_result.cache_manager_get(board.get_hash()) {
                    // Check if we have a sufficient lookup distance
                    let cache_lookahead = cache_result.search_depth - cache_result.move_depth;
                    let current_lookahead = (params.depth_lim + params.extension) - params.depth;
                    let evaluation_valid = cache_lookahead >= current_lookahead;

                    _skip_tt_push = current_lookahead < cache_lookahead; // Don't replace in TT if our lookahead is worse

                    if evaluation_valid {
                        // If the cache is valid manage the cache
                        node_stats.caches_used += 1;
                        match cache_result.flag {
                            BoundType::Exact => {
                                return Ok(SearchOutput {
                                    node_eval: cache_result.evaluation,
                                    best_move: Default::default(),
                                    node_stats,
                                });
                                // return value directly
                            }
                            BoundType::UpperBound => {
                                params.alpha = max(
                                    params.alpha,
                                    cache_result.evaluation.for_colour(board.side_to_move()),
                                )
                            }
                            BoundType::LowerBound => {
                                params.beta = min(
                                    params.beta,
                                    cache_result.evaluation.for_colour(board.side_to_move()),
                                )
                            }
                        };
                        if params.alpha >= params.beta {
                            return Ok(SearchOutput {
                                node_eval: cache_result.evaluation,
                                best_move: Default::default(),
                                node_stats,
                            });
                        }
                        // Populate the PV and cutoff moves
                        // Look at these first even if the cache isn't technically valid
                        cache_pv_move = cache_result.pv_move;
                        cache_cutoff_move = cache_result.cutoff_move;
                    }
                }
            }
        }
        false => {}
    };
    // ===================== Done TT for this node  ===================== //

    // Initial PV
    // If we're given an initial pc move, make sure we search it first
    if let Some(mve) = params.first_search_move {
        cache_pv_move = Some(mve);
        max_move = mve;
    }

    // ===================== Check time             ===================== //
    // Check if we're overruning the time limit (provided of depth isnt so large)
    if params.depth <= consts::MAX_DEPTH_TO_CHECK_TIME
        && params.t_start.elapsed().unwrap() > params.t_lim
    {
        return Err(()); // Throw an error to abort this depth
    }
    // ===================== Done check time        ===================== //

    // Run Extension if needed
    if (params.depth >= (params.depth_lim + params.extension))
        || (board.status() == BoardStatus::Checkmate)
        || (board.status() == BoardStatus::Stalemate)
    {
        if should_extend(&board, params.extension) {
            // We've determined that the search should be extended
            params.extension += 1;
        } else {
            // We're not in check so finish the search
            let mut _blank_move: ChessMove;

            return Ok(SearchOutput {
                node_eval: quiescent_search(&board, params.alpha, params.beta, 0, params.alternate_eval_fn),
                best_move: Default::default(),
                node_stats,
            });

            // return Ok(SearchOutput {
            //     node_eval: evaluate_board(
            //         board,
            //     ),
            //     best_move: Default::default(),
            //     node_stats,
            // });
        }
    }

    // Generate moves
    let child_moves = MoveGen::new_legal(&board);
    // Get length of moves
    let num_moves = child_moves.len();

    let mut sorted_moves =
        ordering::order_moves(child_moves, board, cache_pv_move, cache_cutoff_move); // sort all the moves

    // Initialize with least desirable evaluation
    let mut max_val = match board.side_to_move() {
        Color::White => crate::consts::UNDESIRABLE_EVAL_WHITE,
        Color::Black => crate::consts::UNDESIRABLE_EVAL_BLACK,
    };

    node_stats.all_nodes += sorted_moves.len() as i32;
    let mut cutoff_move = None;

    for mve in &mut sorted_moves {
        let node_evaluation;
        let _best_move: ChessMove;

        let _move_is_cache_move: bool = false; // Flag to check if the move is a cache move or not (avoid rewriting)

        let search_result = find_best_move(
            board.make_move_new(mve),
            SearchParameters {
                depth: params.depth + 1,
                depth_lim: params.depth_lim,
                extension: params.extension,
                alpha: -params.beta,
                beta: -params.alpha,
                color: flip_colour(board.side_to_move()),
                cache: params.cache.clone(),
                t_start: params.t_start,
                t_lim: params.t_lim,
                first_search_move: None,
                alternate_eval_fn: params.alternate_eval_fn.clone(),
            },
        );

        match search_result {
            Ok(result) => {
                node_evaluation = result.node_eval;
                node_stats += result.node_stats;
            }
            Err(e) => match params.depth > 0 {
                true => {
                    // We are not at the root node, let the error bubble up to halt the search
                    return Err(e);
                }
                false => {
                    // We are at the root node, what we don't want to do here is return an error.
                    // This would eliminate any benefit we get from the deepening
                    // Instead, break out of the loop and return the best value we have
                    if max_move == Default::default() {
                        // If we only have the default move just use our current move (shouldnt ever happen)
                        max_move = mve;
                    }
                    break;
                }
            },
        }

        // Replace with best move if we determine the move is the best for our current board side
        if node_evaluation.for_colour(board.side_to_move())
            > max_val.for_colour(board.side_to_move())
        {
            max_val = node_evaluation;
            max_move = mve;
        }

        if consts::DEBUG_MODE {
            println!("Move under consideration {}, number of possible moves {}, evaluation (for colour) {}, depth {}, colour {:#?}", mve, num_moves, node_evaluation.for_colour(board.side_to_move()), params.depth, board.side_to_move())
        }

        params.alpha = max(
            params.alpha,
            node_evaluation.for_colour(board.side_to_move()),
        );

        if params.alpha >= params.beta {
            cutoff_move = Some(mve);
            // Alpha beta cutoff here
            break;
        }
    }

    let node_value = max_val;
    let node_flag: BoundType;
    if node_value.for_colour(board.side_to_move()) <= alpha_orig {
        node_flag = BoundType::UpperBound;
    } else if node_value.for_colour(board.side_to_move()) >= params.beta {
        node_flag = BoundType::LowerBound;
    } else {
        node_flag = BoundType::Exact;
    }

    let node_entry = CacheEntry {
        board_hash: board.get_hash(),
        cachedata: CacheData {
            move_depth: params.depth,
            search_depth: params.depth_lim + params.extension,
            evaluation: node_value,
            flag: node_flag,
            pv_move: Some(max_move),
            cutoff_move: cutoff_move,
        },
    };
    let _ = params.cache.cache_tx.send(node_entry);

    Ok(SearchOutput {
        node_eval: max_val,
        best_move: max_move,
        node_stats,
    })
}

#[cfg(test)]
mod tests {
    // use::super*;

    use std::{
        str::FromStr,
        time::{Duration, SystemTime},
    };

    use parking_lot::RwLock;
    use std::sync::Arc;

    use chess::{Board, ChessMove, Square};

    use crate::{
        managers::{
            cache_manager::{Cache, CacheInputGrouping},
            stats_manager::Statistics,
        },
        utils::{search_interface::SearchParameters, common::EvalFunc},
    };

    use super::find_best_move;

    #[test]
    #[serial_test::serial]
    fn test_for_first_move_issue() {
        let fen = "r1k2b1r/ppp1nNpp/2p5/4P3/5Bb1/2N5/PPP2P1P/R4RK1 b - - 0 1";
        let board = Board::from_str(fen).expect("board should be valid");
        let mve = ChessMove::new(Square::H8, Square::G8, None);

        let _board_b = board.make_move_new(mve);

        let _run_stats = Statistics {
            all_nodes: 0,
            searched_nodes: 0,
            caches_used: 0,
        };

        let t_start = SystemTime::now(); // Initial time before running

        let cache_arc = Arc::new(RwLock::new(Cache::default()));
        let (cache_tx, _cache_rx) = Cache::generate_channel();

        let cache = CacheInputGrouping {
            cache_ref: cache_arc,
            cache_tx,
        };

        let search_res = find_best_move(
            board,
            SearchParameters::<EvalFunc> {
                depth: 0,
                depth_lim: 3,
                extension: 0,
                alpha: i16::MAX - 1,
                beta: i16::MIN + 1,
                color: board.side_to_move(),
                cache,
                t_start: &t_start,
                t_lim: Duration::from_secs(7),
                first_search_move: Some(mve),
                alternate_eval_fn: None,
            },
        )
        .unwrap();
        println!("{:#?}", search_res.best_move.to_string());
        assert_ne!(
            search_res.best_move,
            ChessMove::new(Square::C3, Square::E4, None)
        )
    }

    #[test]
    #[serial_test::serial]
    fn test_performance_final_depth() {
        let fen = "r1k2b1r/ppp1nNpp/2p5/4P3/5Bb1/2N5/PPP2P1P/R4RK1 b - - 0 1";
        let board = Board::from_str(fen).expect("board should be valid");
        let mve = ChessMove::new(Square::H8, Square::G8, None);

        let _board_b = board.make_move_new(mve);

        let _run_stats = Statistics {
            all_nodes: 0,
            searched_nodes: 0,
            caches_used: 0,
        };

        let t_start = SystemTime::now(); // Initial time before running

        let cache_arc = Arc::new(RwLock::new(Cache::default()));
        let (cache_tx, _cache_rx) = Cache::generate_channel();

        let cache = CacheInputGrouping {
            cache_ref: cache_arc,
            cache_tx,
        };

        let search_res = find_best_move(
            board,
            SearchParameters::<EvalFunc> {
                depth: 2,
                depth_lim: 3,
                extension: 0,
                alpha: i16::MAX - 1,
                beta: i16::MIN + 1,
                color: board.side_to_move(),
                cache,
                t_start: &t_start,
                t_lim: Duration::from_secs(7),
                first_search_move: Some(mve),
                alternate_eval_fn: None,
            },
        )
        .unwrap();
        println!("{:#?}", search_res.best_move.to_string());
        assert_ne!(
            search_res.best_move,
            ChessMove::new(Square::C3, Square::E4, None)
        )
    }
}
