use std::time::SystemTime;

use chess::{Board, Color, CacheTable, MoveGen, ChessMove, BoardStatus};
use crate::consts;
use crate::engine::HashtableResultType;
use crate::quiescent::search_captures;
use crate::{engine::{CacheData, Eval, max, flip_colour, Statistics}, ordering};



pub(crate) fn find_best_move(
    board: Board,
    depth: i16,
    depth_lim: i16,
    mut alpha: i16,
    beta: i16,
    color_i: Color,
    stats_data: &mut Statistics,
    cache: &mut CacheTable<CacheData>,
    t_start: &SystemTime
) -> Result<(Eval, ChessMove, [ChessMove; consts::DEPTH_LIM as usize]), ()> {

    // Check if we're overruning the time limit (provided of depth isnt so large)
    if depth <= consts::MAX_DEPTH_TO_CHECK_TIME && t_start.elapsed().unwrap() > consts::TIME_LIM {
        println!("Cancelling search");
        return Err(()) // Throw an error to abort this depth
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
            search_captures(&board, alpha, beta, 0, color_i, cache, depth_lim),
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

    let mut sorted_moves = ordering::order_moves(child_moves, board, cache, false, false, depth, depth_lim); // sort all the moves

    // Initialize with least desirable evaluation
    let mut max_val = match color_i {
        Color::White => crate::consts::UNDESIRABLE_EVAL_WHITE,
        Color::Black => crate::consts::UNDESIRABLE_EVAL_BLACK,
    };

    let mut max_move = sorted_moves[0].chessmove.clone();
    let mut max_line: [ChessMove; consts::DEPTH_LIM as usize] = [max_move; consts::DEPTH_LIM as usize];

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
                (node_evaluation, _best_move, proposed_line) = find_best_move(
                    board.make_move_new(mve),
                    depth + 1,
                    depth_lim,
                    -beta,
                    -alpha,
                    flip_colour(color_i),
                    stats_data,
                    cache,
                    t_start
                )?;

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

        alpha = max(alpha, node_evaluation.for_colour(board.side_to_move()));

        if alpha >= beta {
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

    return Ok((max_val, max_move, max_line));
}