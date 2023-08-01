use std::time::SystemTime;

use crate::consts;
use crate::quiescent::search_captures;
use crate::{
    utils::{flip_colour, max, CacheData, Eval, Statistics, HashtableResultType},
    ordering,
};
use chess::{Board, BoardStatus, CacheTable, ChessMove, Color, MoveGen};

pub fn find_best_move(
    board: Board,
    depth: i16,
    depth_lim: i16,
    alpha: i16,
    beta: i16,
    color_i: Color,
    stats_data: &mut Statistics,
    cache: &mut CacheTable<CacheData>,
    t_start: &SystemTime,
    first_search_move: Option<ChessMove>,
) -> Result<(Eval, ChessMove, [ChessMove; consts::DEPTH_LIM as usize]), ()> {
    let mut alpha_node = alpha;

    // Check if we're overruning the time limit (provided of depth isnt so large)
    if depth <= consts::MAX_DEPTH_TO_CHECK_TIME && t_start.elapsed().unwrap() > consts::TIME_LIM {
        return Err(()); // Throw an error to abort this depth
    }

    if (depth >= depth_lim)
        || (board.status() == BoardStatus::Checkmate)
        || (board.status() == BoardStatus::Stalemate)
    {
        let mut _blank_move: ChessMove;
        let proposed_line: [ChessMove; consts::DEPTH_LIM as usize] =
            [Default::default(); consts::DEPTH_LIM as usize];

        // Note, issues with pruning, does weird things
        return Ok((
            search_captures(&board, alpha_node, beta, 0, color_i, cache, depth_lim),
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

    let mut sorted_moves =
        ordering::order_moves(child_moves, board, cache, false, false, depth, depth_lim); // sort all the moves

    // Initialize with least desirable evaluation
    let mut max_val = match color_i {
        Color::White => crate::consts::UNDESIRABLE_EVAL_WHITE,
        Color::Black => crate::consts::UNDESIRABLE_EVAL_BLACK,
    };

    let mut max_move = sorted_moves[0].chessmove;
    let mut max_line: [ChessMove; consts::DEPTH_LIM as usize] =
        [max_move; consts::DEPTH_LIM as usize];

    // If we get in a move that we must make first, do that before going through the other moves
    if let Some(mve) = first_search_move {
        // Set the PV move to our best move
        max_move = mve;
        // Run a search to the current depth on the PV move
        (max_val, _, max_line) = find_best_move(
            board.make_move_new(mve),
            depth + 1,
            depth_lim,
            -beta,
            -alpha_node,
            flip_colour(color_i),
            stats_data,
            cache,
            t_start,
            None,
        )?;
        // Overwrite the PV move in hash so that we don't need to evaluate it twice
        // (The search routine should catch the fact that we have it already in the cache)
        cache.add(
            board.make_move_new(max_move).get_hash(),
            CacheData {
                move_depth: depth,
                search_depth: depth_lim,
                evaluation: max_val,
                move_type: HashtableResultType::PVMove,
            },
        );
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
                if depth > 0 {
                    (node_evaluation, _best_move, proposed_line) = find_best_move(
                        board.make_move_new(mve),
                        depth + 1,
                        depth_lim,
                        -beta,
                        -alpha_node,
                        flip_colour(color_i),
                        stats_data,
                        cache,
                        t_start,
                        None,
                    )?;
                } else {
                    // We are at the root node, what we don't want to do here is return an error.
                    // This would eliminate any benefit we get from the deepening
                    // Instead, break out of the loop and return the best value we have
                    let search_result = find_best_move(
                        board.make_move_new(mve),
                        depth + 1,
                        depth_lim,
                        -beta,
                        -alpha_node,
                        flip_colour(color_i),
                        stats_data,
                        cache,
                        t_start,
                        None,
                    );

                    match search_result {
                        Ok(result) => (node_evaluation, _best_move, proposed_line) = result,
                        Err(_) => break,
                    }
                }

                // Add move to hash
                cache.add(
                    board.make_move_new(mve).get_hash(),
                    CacheData {
                        move_depth: depth,
                        search_depth: depth_lim,
                        evaluation: node_evaluation,
                        move_type: HashtableResultType::RegularMove,
                    },
                );
            }
        }

        // Update stats
        if consts::SEARCH_INFO {
            stats_data.searched_nodes += 1
        }

        // Replace with best move if we determine the move is the best for our current board side
        if node_evaluation.for_colour(color_i) > max_val.for_colour(color_i) {
            max_val = node_evaluation;
            max_move = mve;
            max_line = proposed_line;
            max_line[depth as usize] = max_move;
        }

        if consts::DEBUG_MODE {
            println!("Move under consideration {}, number of possible moves {}, evaluation (for colour) {}, depth {}, colour {:#?}", mve, num_moves, node_evaluation.for_colour(color_i), depth, color_i)
        }

        alpha_node = max(alpha_node, node_evaluation.for_colour(board.side_to_move()));

        if alpha_node >= beta {
            // Alpha beta cutoff here

            // Record in cache that this is a cutoff move
            cache.add(
                board.make_move_new(mve).get_hash(),
                CacheData {
                    move_depth: depth,
                    search_depth: depth_lim,
                    evaluation: node_evaluation,
                    move_type: HashtableResultType::CutoffMove,
                },
            );

            break;
        }
    }

    // Overwrite the PV move in the hash
    cache.add(
        board.make_move_new(max_move).get_hash(),
        CacheData {
            move_depth: depth,
            search_depth: depth_lim,
            evaluation: max_val,
            move_type: HashtableResultType::PVMove,
        },
    );

    Ok((max_val, max_move, max_line))
}

#[cfg(test)]
mod tests {
    // use::super*;

    use std::{str::FromStr, time::SystemTime};

    use chess::{Board, CacheTable, ChessMove, Color, Square};

    use crate::utils::{CacheData, Eval, HashtableResultType, Statistics};

    use super::find_best_move;

    #[test]
    fn test_bug() {
        let fen = "r1k2b1r/ppp1nNpp/2p5/4P3/5Bb1/2N5/PPP2P1P/R4RK1 b - - 0 1";
        let board = Board::from_str(fen).expect("board should be valid");
        let mve = ChessMove::new(Square::H8, Square::G8, None);

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

        let search_res = find_best_move(
            board,
            0,
            3,
            i16::MAX - 1,
            i16::MIN + 1,
            board.side_to_move(),
            &mut run_stats,
            &mut cache,
            &t_start,
            Some(mve),
        )
        .unwrap();
        println!("{:#?}", search_res.1.to_string());
        assert_ne!(search_res.1, ChessMove::new(Square::C3, Square::E4, None))
    }
}
