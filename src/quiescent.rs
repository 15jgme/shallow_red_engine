use chess::{Board, MoveGen};

use crate::{utils::common::{Eval, abs_eval_from_color, max}, evaluation::evaluate_board, ordering, consts::QUIESENT_LIM, managers::cache_manager::CacheInputGrouping};




pub(crate) fn quiescent_search(
    board: &Board,
    mut alpha: i16,
    beta: i16,
    depth: i16,
    cache: CacheInputGrouping,
    depth_lim: i16,
) -> Eval {

    // Search through all terminal captures
    let stand_pat = evaluate_board(*board);
    if stand_pat.for_colour(board.side_to_move()) >= beta || depth > QUIESENT_LIM {
        return abs_eval_from_color(beta, board.side_to_move());
    }

    alpha = max(alpha, stand_pat.for_colour(board.side_to_move()));

    let capture_moves = MoveGen::new_legal(board);
    let sorted_moves = ordering::order_moves(capture_moves, board.clone(), cache.clone(), true, true, depth, depth_lim); // sort all the moves

    for capture_move_score in sorted_moves {
        let capture_move = capture_move_score.chessmove;
        let score = quiescent_search(
            &board.make_move_new(capture_move),
            -beta,
            -alpha,
            depth + 1,
            cache.clone(),
            depth_lim,
        );

        if score.for_colour(board.side_to_move()) >= beta {
            return abs_eval_from_color(beta, board.side_to_move());
        }

        alpha = max(alpha, score.for_colour(board.side_to_move()));
    }

    abs_eval_from_color(alpha, board.side_to_move())
}
