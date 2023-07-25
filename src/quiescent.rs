use chess::{Board, Color, CacheTable, MoveGen};

use crate::{engine::{CacheData, Eval, abs_eval_from_color, max, flip_colour}, evaluation::evaluate_board, ordering, consts::QUIESENT_LIM};



pub(crate) fn search_captures(
    board: &Board,
    alpha_old: i16,
    beta: i16,
    depth: i16,
    color_i: Color,
    cache: &mut CacheTable<CacheData>,
    depth_lim: i16,
) -> Eval {
    let mut alpha = alpha_old;

    // Search through all terminal captures
    let stand_pat = evaluate_board(*board);
    if stand_pat.for_colour(color_i) >= beta || depth > QUIESENT_LIM {
        return abs_eval_from_color(beta, color_i);
    }

    alpha = max(alpha, stand_pat.for_colour(color_i));

    let capture_moves = MoveGen::new_legal(&board);
    let sorted_moves = ordering::order_moves(capture_moves, *board, cache, true, depth_lim); // sort all the moves

    for capture_move_score in sorted_moves {
        let capture_move = capture_move_score.chessmove;
        let score = search_captures(
            &board.make_move_new(capture_move),
            -beta,
            -alpha,
            depth + 1,
            flip_colour(color_i),
            cache,
            depth_lim,
        );

        if score.for_colour(color_i) >= beta {
            return abs_eval_from_color(beta, color_i);
        }

        alpha = max(alpha, score.for_colour(color_i));
    }

    return abs_eval_from_color(alpha, color_i);
}