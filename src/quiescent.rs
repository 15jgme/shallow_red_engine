use chess::{Board, MoveGen};

use crate::{utils::common::{Eval, abs_eval_from_color}, evaluation::evaluate_board, ordering, consts::QUIESENT_LIM};

fn fetch_sorted_captures(board: &Board) -> std::vec::IntoIter<chess::ChessMove>{
    let mut capture_moves = MoveGen::new_legal(board);
    capture_moves.set_iterator_mask(*board.color_combined(!board.side_to_move())); // Set mask for captures
    ordering::order_moves(capture_moves, *board, None, None) // sort all the moves
}

pub(crate) fn quiescent_search<EvalFunc>(
    board: &Board,
    alpha: i16,
    beta: i16,
    depth: u8,
    alternate_eval_fn: Option<EvalFunc>,
) -> Eval where EvalFunc: Fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) -> i16 + Clone {
    let mut alpha = alpha;
    // Search through all terminal captures
    let stand_pat = evaluate_board(*board, alternate_eval_fn.clone());

    if stand_pat.for_colour(board.side_to_move()) >= beta{
        return abs_eval_from_color(beta, board.side_to_move());
    }

    if depth > QUIESENT_LIM{
        return stand_pat;
    }

    let sorted_moves = fetch_sorted_captures(board);

    if alpha < stand_pat.for_colour(board.side_to_move()) {
        alpha = stand_pat.for_colour(board.side_to_move());
    }

    for capture_move in sorted_moves {
        let score = quiescent_search(
            &board.make_move_new(capture_move),
            -beta,
            -alpha,
            depth + 1,
            alternate_eval_fn.clone()
        );

        if score.for_colour(board.side_to_move()) >= beta {
            return abs_eval_from_color(beta, board.side_to_move());
        }
        if stand_pat.for_colour(board.side_to_move()) > alpha {
            alpha = stand_pat.for_colour(board.side_to_move());
        }
    }
    abs_eval_from_color(alpha, board.side_to_move())
}

#[cfg(test)]
mod tests{
    use std::str::FromStr;

    use chess::{Board, ChessMove, Square};

    use crate::utils::common::EvalFunc;

    use super::{fetch_sorted_captures, quiescent_search};

    #[test]
    #[serial_test::serial]
    fn test_caputes_only(){
        let board_init: Board = Default::default();
        let mut sorted_cap = fetch_sorted_captures(&board_init);
        assert_eq!(sorted_cap.next(), None); // Confirm we are using only captures!

        let board_eg: Board = Board::from_str("8/3K4/8/8/8/8/3R4/3k4 b - - 0 1").unwrap();
        let mut sorted_cap = fetch_sorted_captures(&board_eg);
        assert_eq!(sorted_cap.next(), Some(ChessMove::new(Square::D1, Square::D2, None)));
        assert_eq!(sorted_cap.next(), None);
    }

    #[test]
    #[serial_test::serial]
    fn test_quiescent_basic(){
        let board_eg: Board = Board::from_str("8/3K4/8/8/8/8/3R4/3k4 b - - 0 1").unwrap();
        let q_res = quiescent_search::<EvalFunc>(&board_eg, i16::min_value() + 1, i16::max_value() - 1, 0, None);
        println!("{:#?}", q_res);
    }
}