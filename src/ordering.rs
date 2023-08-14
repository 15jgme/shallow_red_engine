use crate::managers::cache_manager::BoundType;
use crate::utils::common::Eval;
use chess::{Board, ChessMove, MoveGen, Piece};
use itertools::Itertools;

fn get_piece_weight(piece: Piece) -> i16 {
    // Return the estimated value of a piece
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 300,
        Piece::Bishop => 320,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 2500,
    }
}

#[derive(PartialEq)]
pub(crate) struct RetreivedCacheData {
    pub(crate) evaluation: Eval,
    pub(crate) flag: BoundType,
}

pub(crate) fn order_moves(
    moves: MoveGen,
    board: Board,
    pv_move: Option<ChessMove>,
    cutoff_move: Option<ChessMove>,
) -> std::vec::IntoIter<chess::ChessMove> {
    let sorted_moves = moves.sorted_by_cached_key(|mve| {
        // Check if this move is our PV move
        if let Some(pv) = pv_move {
            if *mve == pv {
                return i16::MIN;
            }
        }

        // Check if this move is our cutoff move
        if let Some(coff) = cutoff_move {
            if *mve == coff {
                return i16::MIN + 1;
            }
        }

        // If we have a promotion sort it very highly
        if let Some(promotion) = mve.get_promotion() {
            return i16::MIN + 2 + (get_piece_weight(Piece::Queen) - get_piece_weight(promotion));
        }

        // Check if move is a non-capture
        let destination_piece = board.piece_on(mve.get_dest());
        let source_piece = board.piece_on(mve.get_source());

        match (source_piece, destination_piece) {
            (Some(source), Some(dest)) => {
                let mvv_lva = get_piece_weight(dest) - get_piece_weight(source);
                return -(mvv_lva + get_piece_weight(Piece::King)); // Sort is ascending, provide a boost for all the captures
            }
            (_, _) => return 0,
        };
    });
    return sorted_moves;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::order_moves;
    use chess::{Board, ChessMove, MoveGen, Square};
    #[test]
    fn test_simple_ordering() {
        let board: Board = Board::from_str("2rk4/8/8/8/1B6/3r4/4P3/3Kb3 w - - 0 1").unwrap();
        let moves = MoveGen::new_legal(&board);
        let mut sorted_moves = order_moves(moves, board, None, None);

        assert_eq!(
            sorted_moves.next(),
            Some(ChessMove::new(Square::E2, Square::D3, None))
        );
        assert_eq!(
            sorted_moves.next(),
            Some(ChessMove::new(Square::D1, Square::E1, None))
        );
        assert_eq!(
            sorted_moves.next(),
            Some(ChessMove::new(Square::B4, Square::D2, None))
        );
    }

    #[test]
    fn test_pv_cv_ordering() {
        let board: Board = Board::from_str("2rk4/8/8/8/1B6/3r4/Q3P3/3Kb3 w - - 0 1").unwrap();
        let pv_move = ChessMove::new(Square::A2, Square::D2, None);
        let co_move = ChessMove::new(Square::D1, Square::E1, None);
        let moves = MoveGen::new_legal(&board);
        let mut sorted_moves = order_moves(moves, board, Some(pv_move), Some(co_move));

        assert_eq!(sorted_moves.next(), Some(pv_move));
        assert_eq!(sorted_moves.next(), Some(co_move));
        assert_eq!(
            sorted_moves.next(),
            Some(ChessMove::new(Square::E2, Square::D3, None))
        );
    }
}
