use crate::consts;
use crate::managers::cache_manager::{CacheData, CacheEntry, HashtableResultType};
use crate::managers::stats_manager::{Statistics, StatisticsInputGrouping};
use crate::quiescent::quiescent_search;
use crate::utils::search_interface::SearchParameters;
use crate::{
    ordering,
    utils::common::{flip_colour, max, Eval},
};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};

pub fn find_best_move(
    board: Board,
    params: SearchParameters,
    stats_data: &mut Statistics,
) -> Result<(Eval, ChessMove, [ChessMove; consts::DEPTH_LIM as usize]), ()> {
    let mut alpha_node = params.alpha;

    // Check if we're overruning the time limit (provided of depth isnt so large)
    if params.depth <= consts::MAX_DEPTH_TO_CHECK_TIME
        && params.t_start.elapsed().unwrap() > consts::TIME_LIM
    {
        return Err(()); // Throw an error to abort this depth
    }

    if (params.depth >= params.depth_lim)
        || (board.status() == BoardStatus::Checkmate)
        || (board.status() == BoardStatus::Stalemate)
    {
        let mut _blank_move: ChessMove;
        let proposed_line: [ChessMove; consts::DEPTH_LIM as usize] =
            [Default::default(); consts::DEPTH_LIM as usize];

        return Ok((
            quiescent_search(
                &board,
                alpha_node,
                params.beta,
                0,
                params.color,
                params.cache.clone(),
                params.depth_lim,
            ),
            Default::default(),
            proposed_line,
        ));

        // return Ok((
        //     crate::evaluation::evaluate_board(board),
        //     Default::default(),
        //     proposed_line,
        // ));
    }

    // Generate moves
    let child_moves = MoveGen::new_legal(&board);
    // Get length of moves
    let num_moves = child_moves.len();

    if consts::SEARCH_INFO {
        stats_data.all_nodes += num_moves as i32
    }

    let mut sorted_moves = ordering::order_moves(
        child_moves,
        board,
        params.cache.clone(),
        false,
        false,
        params.depth,
        params.depth_lim,
    ); // sort all the moves

    // Initialize with least desirable evaluation
    let mut max_val = match params.color {
        Color::White => crate::consts::UNDESIRABLE_EVAL_WHITE,
        Color::Black => crate::consts::UNDESIRABLE_EVAL_BLACK,
    };

    let mut max_move = sorted_moves[0].chessmove;
    let mut max_line: [ChessMove; consts::DEPTH_LIM as usize] =
        [max_move; consts::DEPTH_LIM as usize];

    // If we get in a move that we must make first, do that before going through the other moves
    if let Some(mve) = params.first_search_move {
        // Set the PV move to our best move
        max_move = mve;
        // Run a search to the current depth on the PV move
        // jackson dedbugging todo. BREAKS HERE (search works when we use )
        (max_val, _, max_line) = find_best_move(
            board.make_move_new(mve),
            SearchParameters {
                depth: params.depth + 1,
                depth_lim: params.depth_lim,
                alpha: -params.beta,
                beta: -alpha_node,
                color: flip_colour(params.color),
                stats: StatisticsInputGrouping{},
                cache: params.cache.clone(),
                t_start: params.t_start,
                first_search_move: None,
            },
            stats_data,
        )?;
        // Overwrite the PV move in hash so that we don't need to evaluate it twice
        // (The search routine should catch the fact that we have it already in the cache)
        let _ = params.cache.cache_tx.send(CacheEntry {
            board: board.make_move_new(max_move),
            cachedata: CacheData {
                move_depth: params.depth,
                search_depth: params.depth_lim,
                evaluation: max_val,
                move_type: HashtableResultType::PVMove,
            },
        });
    }

    for weighted_move in &mut sorted_moves {
        let mve = weighted_move.chessmove;

        let (node_evaluation, _best_move, proposed_line);

        match weighted_move.evaluation {
            Some(eval) => {
                // We've found this move in the current search no need to assess
                node_evaluation = eval;
                _best_move = Default::default();
                proposed_line = [Default::default(); consts::DEPTH_LIM as usize];
                stats_data.caches_used += 1;
            }
            None => {
                if params.depth > 0 {
                    (node_evaluation, _best_move, proposed_line) = find_best_move(
                        board.make_move_new(mve),
                        SearchParameters {
                            depth: params.depth + 1,
                            depth_lim: params.depth_lim,
                            alpha: -params.beta,
                            beta: -alpha_node,
                            color: flip_colour(params.color),
                            stats: StatisticsInputGrouping {},
                            cache: params.cache.clone(),
                            t_start: params.t_start,
                            first_search_move: None,
                        },
                        stats_data,
                    )?;
                } else {
                    // We are at the root node, what we don't want to do here is return an error.
                    // This would eliminate any benefit we get from the deepening
                    // Instead, break out of the loop and return the best value we have

                    let search_result = find_best_move(
                        board.make_move_new(mve),
                        SearchParameters {
                            depth: params.depth + 1,
                            depth_lim: params.depth_lim,
                            alpha: -params.beta,
                            beta: -alpha_node,
                            color: flip_colour(params.color),
                            stats: StatisticsInputGrouping {},
                            cache: params.cache.clone(),
                            t_start: params.t_start,
                            first_search_move: None,
                        },
                        stats_data,
                    );

                    match search_result {
                        Ok(result) => (node_evaluation, _best_move, proposed_line) = result,
                        Err(_) => break,
                    }
                }

                // Add move to hash
                let _ = params.cache.cache_tx.send(CacheEntry {
                    board: board.make_move_new(mve),
                    cachedata: CacheData {
                        move_depth: params.depth,
                        search_depth: params.depth_lim,
                        evaluation: node_evaluation,
                        move_type: HashtableResultType::RegularMove,
                    },
                });
            }
        }

        // Update stats
        if consts::SEARCH_INFO {
            stats_data.searched_nodes += 1
        }

        // Replace with best move if we determine the move is the best for our current board side
        if node_evaluation.for_colour(params.color) > max_val.for_colour(params.color) {
            max_val = node_evaluation;
            max_move = mve;
            max_line = proposed_line;
            max_line[params.depth as usize] = max_move;
        }

        if consts::DEBUG_MODE {
            println!("Move under consideration {}, number of possible moves {}, evaluation (for colour) {}, depth {}, colour {:#?}", mve, num_moves, node_evaluation.for_colour(params.color), params.depth, params.color)
        }

        alpha_node = max(alpha_node, node_evaluation.for_colour(board.side_to_move()));

        if alpha_node >= params.beta {
            // Alpha beta cutoff here

            // Record in cache that this is a cutoff move
            let _ = params.cache.cache_tx.send(CacheEntry {
                board: board.make_move_new(mve),
                cachedata: CacheData {
                    move_depth: params.depth,
                    search_depth: params.depth_lim,
                    evaluation: node_evaluation,
                    move_type: HashtableResultType::CutoffMove,
                },
            });
            break;
        }
    }

    // Overwrite the PV move in the hash
    let _ = params.cache.cache_tx.send(    CacheEntry{
        board: board.make_move_new(max_move),
        cachedata: CacheData {
            move_depth: params.depth,
            search_depth: params.depth_lim,
            evaluation: max_val,
            move_type: HashtableResultType::PVMove,
        },
    });


    Ok((max_val, max_move, max_line))
}

#[cfg(test)]
mod tests {
    // use::super*;

    use std::{str::FromStr, time::SystemTime, sync::{Arc, RwLock}};

    use chess::{Board, ChessMove, Square};

    use crate::{
        managers::{
            cache_manager::{Cache, CacheInputGrouping},
            stats_manager::{Statistics, StatisticsInputGrouping},
        },
        utils::search_interface::SearchParameters,
    };

    use super::find_best_move;

    #[test]
    fn test_for_first_move_issue() {
        let fen = "r1k2b1r/ppp1nNpp/2p5/4P3/5Bb1/2N5/PPP2P1P/R4RK1 b - - 0 1";
        let board = Board::from_str(fen).expect("board should be valid");
        let mve = ChessMove::new(Square::H8, Square::G8, None);

        let _board_b = board.make_move_new(mve);

        let mut run_stats = Statistics {
            all_nodes: 0,
            searched_nodes: 0,
            caches_used: 0,
            time_ms: 0.0,
            depth_reached: 1,
        };

        let t_start = SystemTime::now(); // Initial time before running

        let cache_arc = Arc::new(RwLock::new(Cache::default()));
        let (cache_tx, _cache_rx) = Cache::generate_channel();

        let cache = CacheInputGrouping{ cache_ref: cache_arc, cache_tx };

        let search_res = find_best_move(
            board,
            SearchParameters {
                depth: 0,
                depth_lim: 3,
                alpha: i16::MAX - 1,
                beta: i16::MIN + 1,
                color: board.side_to_move(),
                stats: StatisticsInputGrouping{},
                cache,
                t_start: &t_start,
                first_search_move: Some(mve),
            },&mut run_stats
        )
        .unwrap();
        println!("{:#?}", search_res.1.to_string());
        assert_ne!(search_res.1, ChessMove::new(Square::C3, Square::E4, None))
    }
}
