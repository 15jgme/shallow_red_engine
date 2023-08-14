use chess::{BitBoard, Board, BoardStatus, Piece, EMPTY};

use crate::consts;

pub(crate) fn should_extend(board: &Board, current_extension: u8) -> bool {
    // Returns true if the search should be extended
    // let bb_7th_rank: BitBoard = BitBoard(0xff000000000000);
    // let bb_1st_rank: BitBoard = BitBoard(0b1111111100000000);

    let should_extend_for_check = *board.checkers() != EMPTY;

    // let should_extend_for_promote_black =
    //     (board.pieces(Piece::Pawn) & board.color_combined(chess::Color::Black) & bb_1st_rank)
    //         .popcnt()
    //         > 0;

    // let should_extend_for_promote_white =
    //     (board.pieces(Piece::Pawn) & board.color_combined(chess::Color::White) & bb_7th_rank)
    //         .popcnt()
    //         > 0;

    // let should_extend_for_promote = match board.side_to_move(){
    //     chess::Color::White => should_extend_for_promote_white,
    //     chess::Color::Black => should_extend_for_promote_black,
    // };

    let not_constrained =
        current_extension < consts::EXTENSION_LIM && board.status() == BoardStatus::Ongoing;

    // (should_extend_for_check || should_extend_for_promote)
    //     && not_constrained
    (should_extend_for_check) && not_constrained
}

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use chess::Board;

//     use crate::{consts, extensions::should_extend};

//     #[test]
//     fn test_default_board() {
//         let board = Board::default();
//         assert!(!should_extend(&board, 0))
//     }

//     #[test]
//     fn test_white_pawn() {
//         let board = Board::from_str("3K4/6P1/8/8/8/8/8/3k4 w - - 0 1").unwrap();
//         assert!(should_extend(&board, 0))
//     }

//     #[test]
//     fn test_white_pawn_blacks_turn() {
//         let board = Board::from_str("3K4/6P1/8/8/8/8/8/3k4 b - - 0 1").unwrap();
//         assert!(!should_extend(&board, 0))
//     }

//     #[test]
//     fn test_black_pawn() {
//         let board = Board::from_str("3K4/8/8/8/8/8/6p1/3k4 b - - 0 1").unwrap();
//         assert!(should_extend(&board, 0))
//     }

//     #[test]
//     fn test_black_pawn_whites_turn() {
//         let board = Board::from_str("3K4/8/8/8/8/8/6p1/3k4 w - - 0 1").unwrap();
//         assert!(!should_extend(&board, 0))
//     }

//     #[test]
//     fn test_limit() {
//         let board = Board::from_str("3K4/8/8/8/8/8/6p1/3k4 b - - 0 1").unwrap();
//         assert!(!should_extend(&board, consts::EXTENSION_LIM))
//     }

//     #[test]
//     fn test_check() {
//         let board = Board::from_str("3K4/8/8/8/6Q1/8/8/3k4 b - - 0 1").unwrap();
//         assert!(should_extend(&board, 0))
//     }
// }
