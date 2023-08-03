// Piece Square Tables

use chess::{Color, Piece, Square};

use crate::utils::common::{Eval, abs_eval_from_color};
use crate::gamestate::GameState;

const BOARD_LEN: usize = 64;

// Boards are expressed from Black's perspective, to improve legibility, flip these grids at some point

// Pawn ♙
const PAWN_PSQT: [i16; BOARD_LEN] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10,-20,-20, 10, 10,  5,
    5, -5,-10,  0,  0,-10, -5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
    0,  0,  0,  0,  0,  0,  0,  0,
];

// Bishop ♗
const BISHOP_PSQT: [i16; BOARD_LEN] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20, 
];

// Rook ♖
const ROOK_PSQT: [i16; BOARD_LEN]  = [
    0,  0,  0,  5,  5,  0,  0,  0,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,  
    5, 10, 10, 10, 10, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0, 
];

// Knight ♘
const KNIGHT_PSQT: [i16; BOARD_LEN]  = [
    -50,-40,-30,-30,-30,-30,-40,-50, 
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,                       
];

// King ♔
const KING_PSQT_MG: [i16; BOARD_LEN] = [
    20, 30, 10,  0,  0, 10, 30, 20,
    20, 20,  0,  0,  0,  0, 20, 20,
   -10,-20,-20,-20,-20,-20,-20,-10,
   -20,-30,-30,-40,-40,-30,-30,-20,
   -30,-40,-40,-50,-50,-40,-40,-30,
   -30,-40,-40,-50,-50,-40,-40,-30,   
   -30,-40,-40,-50,-50,-40,-40,-30,
   -30,-40,-40,-50,-50,-40,-40,-30,
];

const KING_PSQT_EG: [i16; BOARD_LEN] = [
    -50,-30,-30,-30,-30,-30,-30,-50,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -50,-40,-30,-20,-20,-30,-40,-50,
];

// Queen ♕
const QUEEN_PSQT: [i16; BOARD_LEN] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
      0,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5, 
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,   
    -20,-10,-10, -5, -5,-10,-10,-20,            
];

pub(crate) fn get_psqt_score(piece: Piece, color: Color, square: Square, gamestate: GameState) -> Eval {

    let probe_index = match color {
        Color::White => square.to_index(),
        Color::Black => (BOARD_LEN - 1) - square.to_index(),
    };

    let eval_abs = match piece {
        Piece::Pawn => PAWN_PSQT[probe_index],
        Piece::Knight => KNIGHT_PSQT[probe_index],
        Piece::Bishop => BISHOP_PSQT[probe_index],
        Piece::Rook => ROOK_PSQT[probe_index],
        Piece::Queen => QUEEN_PSQT[probe_index],
        Piece::King => match gamestate {
            GameState::_Opening => KING_PSQT_MG[probe_index],
            GameState::Middle => KING_PSQT_MG[probe_index],
            GameState::End => KING_PSQT_EG[probe_index],
        } ,
    };

    abs_eval_from_color(eval_abs, color)
}

#[cfg(test)]
mod tests {
    use chess::{Square, Color, Piece};
    use crate::gamestate::GameState;

    use super::get_psqt_score;

    #[test]
    fn test_pawn(){
        // White pawn A1
        assert_eq!(get_psqt_score(Piece::Pawn, Color::White, Square::A2, GameState::Middle).for_colour(Color::White), 5);

        // White pawn G6
        assert_eq!(get_psqt_score(Piece::Pawn, Color::White, Square::G6, GameState::Middle).for_colour(Color::White), 10);

        // Confirm symmetry

        // Black pawn G8
        assert_eq!(get_psqt_score(Piece::Pawn, Color::Black, Square::H7, GameState::Middle).for_colour(Color::Black), 5);

        // Black pawn B3
        assert_eq!(get_psqt_score(Piece::Pawn, Color::Black, Square::B3, GameState::Middle).for_colour(Color::Black), 10);
    }

    #[test]
    fn test_bishop(){
        // White bishop A6
        assert_eq!(get_psqt_score(Piece::Bishop, Color::White, Square::A6, GameState::Middle).for_colour(Color::White), -10);

        // Black bishop G5
        assert_eq!(get_psqt_score(Piece::Bishop, Color::Black, Square::G5, GameState::Middle).for_colour(Color::Black), 0);
    }

    #[test]
    fn test_rook(){
        // White rook A6
        assert_eq!(get_psqt_score(Piece::Rook, Color::White, Square::A6, GameState::Middle).for_colour(Color::White), -5);

        // Black rook G5
        assert_eq!(get_psqt_score(Piece::Rook, Color::Black, Square::G5, GameState::Middle).for_colour(Color::Black), 0);
    }

    #[test]
    fn test_knight(){
        // White knight A6
        assert_eq!(get_psqt_score(Piece::Knight, Color::White, Square::A6, GameState::Middle).for_colour(Color::White), -30);

        // Black knight G5
        assert_eq!(get_psqt_score(Piece::Knight, Color::Black, Square::G5, GameState::Middle).for_colour(Color::Black), 5);
    }

    #[test]
    fn test_king(){
        // White king A6
        assert_eq!(get_psqt_score(Piece::King, Color::White, Square::A6, GameState::Middle).for_colour(Color::White), -30);

        // Black king G5
        assert_eq!(get_psqt_score(Piece::King, Color::Black, Square::G5, GameState::Middle).for_colour(Color::Black), -30);

        // Starting position white king
        assert_eq!(get_psqt_score(Piece::King, Color::White, Square::E1, GameState::Middle).for_colour(Color::White), 0);

        // Starting position black king
        assert_eq!(get_psqt_score(Piece::King, Color::Black, Square::E8, GameState::Middle).for_colour(Color::Black), 0);       
    }

    #[test]
    fn test_endgame_king(){
        // White king A6
        assert_eq!(get_psqt_score(Piece::King, Color::White, Square::A6, GameState::End).for_colour(Color::White), -30);

        // Black king G5
        assert_eq!(get_psqt_score(Piece::King, Color::Black, Square::G5, GameState::End).for_colour(Color::Black), -10);

        // Starting position white king
        assert_eq!(get_psqt_score(Piece::King, Color::White, Square::E1, GameState::End).for_colour(Color::White), -30);

        // Starting position black king
        assert_eq!(get_psqt_score(Piece::King, Color::Black, Square::E8, GameState::End).for_colour(Color::Black), -30);          
    }

    #[test]
    fn test_queen(){
        // White queen A6
        assert_eq!(get_psqt_score(Piece::Queen, Color::White, Square::A6, GameState::Middle).for_colour(Color::White), -10);

        // Black queen G5
        assert_eq!(get_psqt_score(Piece::Queen, Color::Black, Square::G5, GameState::Middle).for_colour(Color::Black), 0);
    }
}