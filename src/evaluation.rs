use chess::{Board, BoardStatus, Color, Piece};
use crate::engine::Eval;

pub(crate) fn evaluate_board(board: Board) -> Eval {
    // Returns the current score on the board where white winning is positive and black winning is negative

    match board.status() {
        BoardStatus::Checkmate => {
            // We are always in checkmate with the current side to move
            // Since checkmate ends the game, we only need to asses it once
            // Since we assess after a move, it is safe to check at the child node level
            match board.side_to_move() {
                Color::White => return Eval{score: i16::min_value() + 1},
                Color::Black => return Eval{score: i16::max_value() - 1},
            }
        }
        BoardStatus::Stalemate => {
            return Eval{score: 0}; // Stalemate is a draw game
        }
        BoardStatus::Ongoing => {
            // List of values
            let v_pawn: i16 = 100;
            let v_knight: i16 = 300;
            let v_bishop: i16 = 300;
            let v_rook: i16 = 500;
            let v_queen: i16 = 900;

            let v_king: i16 = i16::MAX; // Temporary, ensure that the king is super valuable

            let mut score: i16 = 0;

            let black_pawns =
                (board.pieces(Piece::Pawn) & board.color_combined(Color::Black)).popcnt() as i16;
            let white_pawns =
                (board.pieces(Piece::Pawn) & board.color_combined(Color::White)).popcnt() as i16;

            let black_knight =
                (board.pieces(Piece::Knight) & board.color_combined(Color::Black)).popcnt() as i16;
            let white_knight =
                (board.pieces(Piece::Knight) & board.color_combined(Color::White)).popcnt() as i16;

            let black_bishop =
                (board.pieces(Piece::Bishop) & board.color_combined(Color::Black)).popcnt() as i16;
            let white_bishop =
                (board.pieces(Piece::Bishop) & board.color_combined(Color::White)).popcnt() as i16;

            let black_rook =
                (board.pieces(Piece::Rook) & board.color_combined(Color::Black)).popcnt() as i16;
            let white_rook =
                (board.pieces(Piece::Rook) & board.color_combined(Color::White)).popcnt() as i16;

            let black_queen =
                (board.pieces(Piece::Queen) & board.color_combined(Color::Black)).popcnt() as i16;
            let white_queen =
                (board.pieces(Piece::Queen) & board.color_combined(Color::White)).popcnt() as i16;

            let black_king =
                (board.pieces(Piece::King) & board.color_combined(Color::Black)).popcnt() as i16;
            let white_king =
                (board.pieces(Piece::King) & board.color_combined(Color::White)).popcnt() as i16;

            score += (white_pawns - black_pawns) * v_pawn;
            score += (white_knight - black_knight) * v_knight;
            score += (white_bishop - black_bishop) * v_bishop;
            score += (white_rook - black_rook) * v_rook;
            score += (white_queen - black_queen) * v_queen;
            score += (white_king - black_king) * v_king; // Temporary

            return Eval{score: score};
        }
    }
}
