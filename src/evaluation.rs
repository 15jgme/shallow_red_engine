use crate::gamestate;
use crate::psqt::get_psqt_score;
use crate::utils::common::{abs_eval_from_color, max, min};
use crate::{gamestate::GameState, utils::common::Eval};
use chess::{Board, BoardStatus, Color, Piece, Square};

fn evaluate_board_material(board: &Board) -> Eval {
    // List of values
    let v_pawn: i16 = 100;
    let v_knight: i16 = 300;
    let v_bishop: i16 = 320;
    let v_rook: i16 = 500;
    let v_queen: i16 = 900;

    let v_king: i16 = i16::MAX; // Temporary, ensure that the king is super valuable

    let mut score: i16 = 0;

    // Material
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

    Eval { score }
}

fn evaluate_board_psqt(board: &Board, gamestate: GameState) -> Eval {
    // Piece-square table

    let mut psqt_eval: Eval = Eval { score: 0 };

    // Iterate through white pieces
    for sq in *board.color_combined(Color::White) {
        // We should expect to find a piece but just to confirm
        match board.piece_on(sq) {
            Some(piece_on_sq) => {
                psqt_eval += get_psqt_score(piece_on_sq, Color::White, sq, gamestate)
            }
            None => {
                println!("No piece found when expected, white. Square {}", sq)
            }
        }
    }

    // Iterate through black pieces
    for sq in *board.color_combined(Color::Black) {
        // We should expect to find a piece but just to confirm
        match board.piece_on(sq) {
            Some(piece_on_sq) => {
                psqt_eval += get_psqt_score(piece_on_sq, Color::Black, sq, gamestate)
            }
            None => {
                println!("No piece found when expected, black. Square {}", sq)
            }
        }
    }

    psqt_eval
}

fn chebyshev_dist(sq_1: Square, sq_2: Square) -> i16 {
    let rank_diff =
        Square::get_rank(&sq_1).to_index() as i16 - Square::get_rank(&sq_2).to_index() as i16;
    let file_diff =
        Square::get_file(&sq_1).to_index() as i16 - Square::get_file(&sq_2).to_index() as i16;
    max(rank_diff.abs(), file_diff.abs())
}

fn edge_distance(sq: Square) -> i16 {
    let rank = sq.get_rank().to_index();
    let file = sq.get_file().to_index();

    let to_nearest_rank = min(rank - 0, 7 - rank);
    let to_nearest_file = min(file - 0, 7 - file);
    min(to_nearest_file, to_nearest_rank) as i16
}

fn endgame_king_heuristics(board: &Board, gamestate: GameState, up_material: bool) -> Eval {
    match gamestate {
        GameState::_Opening => Eval { score: 0 },
        GameState::Middle => Eval { score: 0 },
        GameState::End => {
            // Get squares of both kings
            let king_sq_w = board.king_square(Color::White);
            let king_sq_b = board.king_square(Color::Black);
            let king_distance_bonus = if up_material {
                // Compute the king distance heuristic
                let dist = chebyshev_dist(king_sq_w, king_sq_b);

                let dist_weight = 10; // Want to be as close as possible to enemy king to cut it off
                let score = (5 - dist) * dist_weight;
                abs_eval_from_color(score, board.side_to_move())
            } else {
                abs_eval_from_color(0, board.side_to_move())
            };

            // Compute the king position heuristic
            let (opponent_king_sq, self_king_sq) = match board.side_to_move() {
                Color::White => (king_sq_b, king_sq_w),
                Color::Black => (king_sq_w, king_sq_b),
            };

            let position_weight = 5;
            let king_position_diff =
                position_weight * (edge_distance(self_king_sq) - edge_distance(opponent_king_sq));
            let king_position_bonus = abs_eval_from_color(king_position_diff, board.side_to_move());
            king_distance_bonus + king_position_bonus
        }
    }
}

fn up_substantial_material(material_eval: Eval, side_to_move: Color) -> bool {
    // Returns a boolean depending whether or not we think we're up notable material
    let score = material_eval.for_colour(side_to_move);
    score >= 300 // Don't worry about pawns but a minor piece could do it
}

pub fn evaluate_board(board: Board) -> Eval {
    // Returns the current score on the board where white winning is positive and black winning is negative

    let current_gamestate: GameState = gamestate::gamestate(&board); // Get the current gamestate

    match board.status() {
        BoardStatus::Checkmate => {
            // We are always in checkmate with the current side to move
            // Since checkmate ends the game, we only need to asses it once
            // Since we assess after a move, it is safe to check at the child node level
            abs_eval_from_color(i16::MIN + 1, board.side_to_move())
        }
        BoardStatus::Stalemate => {
            Eval { score: 0 } // Stalemate is a draw game
        }
        BoardStatus::Ongoing => {
            let material_eval = evaluate_board_material(&board);
            let psqt_eval = evaluate_board_psqt(&board, current_gamestate);
            let up_material = up_substantial_material(material_eval, board.side_to_move());
            let king_eg_eval = endgame_king_heuristics(&board, current_gamestate, up_material);

            material_eval + psqt_eval + king_eg_eval
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::common::Eval;
    use chess::Board;

    #[test]
    fn test_default_board_material() {
        let initial_board = Board::default();
        assert_eq!(evaluate_board_material(&initial_board), Eval { score: 0 })
    }

    #[test]
    fn test_default_board_psqt() {
        let initial_board = Board::default();
        assert_eq!(
            evaluate_board_psqt(&initial_board, GameState::Middle),
            Eval { score: 0 }
        )
    }

    #[test]
    fn test_default_board() {
        let initial_board = Board::default();
        assert_eq!(evaluate_board(initial_board), Eval { score: 0 })
    }

    #[test]
    fn test_chebyshev_dist() {
        assert_eq!(chebyshev_dist(Square::F6, Square::F5), 1);
        assert_eq!(chebyshev_dist(Square::F6, Square::B1), 5);
        assert_eq!(chebyshev_dist(Square::F6, Square::C7), 3);
    }

    #[test]
    fn test_edge_distance() {
        assert_eq!(edge_distance(Square::A7), 0);
        assert_eq!(edge_distance(Square::D5), 3);
        assert_eq!(edge_distance(Square::E1), 0);
        assert_eq!(edge_distance(Square::E3), 2);
    }
}
