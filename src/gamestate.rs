// This file contains utilities to monitor the game state (opening/middle/endgame)

use chess::{Board, Piece, Color};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum GameState {
    _Opening,
    Middle,
    End,
}

pub fn gamestate(board: &Board) -> GameState {

    let white_has_queen = (board.pieces(Piece::Queen) & board.color_combined(Color::White)).popcnt() >= 1;
    let black_has_queen = (board.pieces(Piece::Queen) & board.color_combined(Color::Black)).popcnt() >= 1;
    
    let neither_side_has_a_queen =  !white_has_queen && !black_has_queen;

    // If a side has a queen, but only one minor piece left we can also consider this endgame
    let white_has_multiple_minor = ((board.pieces(Piece::Knight) | board.pieces(Piece::Bishop)) & board.color_combined(Color::White)).popcnt() > 1;
    let black_has_multiple_minor = ((board.pieces(Piece::Knight) | board.pieces(Piece::Bishop)) & board.color_combined(Color::Black)).popcnt() > 1;

    // If neither side has a queen, or if either colour has a queen BUT only has one minor piece we declare endgame
    if neither_side_has_a_queen || ((!white_has_multiple_minor && white_has_queen) || (!black_has_multiple_minor && black_has_queen)) {
        return GameState::End
    }

    // Otherwise middle game, TODO add opening
    GameState::Middle
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;
    use chess::Board;

    #[test]
    fn test_endgame_kings(){
        let fen = "8/K7/4P3/8/8/8/4p3/k7 w - - 0 1";
        let board: Board = Board::from_str(fen).unwrap();
        assert_eq!(gamestate(&board), GameState::End)
    }

    #[test]
    fn test_endgame_multipiece(){
        let fen = "8/KQ4n1/4P3/1R6/3r4/8/4p1N1/kq6 w - - 0 1";
        let board: Board = Board::from_str(fen).unwrap();
        assert_eq!(gamestate(&board), GameState::End)
    }

    #[test]
    fn test_asymetric_endgame_middlepiece(){
        let fen = "8/KQ6/4P3/1R6/8/8/6N1/k7 w - - 0 1";
        let board: Board = Board::from_str(fen).unwrap();
        assert_eq!(gamestate(&board), GameState::End)       
    }

    #[test]
    fn test_middlegame(){
        let fen = "7B/KQ6/4P3/1R6/8/8/k5N1/1q1nr2b w - - 0 1";
        let board: Board = Board::from_str(fen).unwrap();
        assert_eq!(gamestate(&board), GameState::Middle)   
    }

    #[test]
    fn test_asymmetric_middlegame(){
        let fen = "7B/K7/8/8/8/8/k7/1q1nr2b w - - 0 1";
        let board: Board = Board::from_str(fen).unwrap();
        assert_eq!(gamestate(&board), GameState::Middle)   
    }
}