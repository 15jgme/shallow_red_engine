use std::cmp::Ordering;

// use itertools::Itertools;

use chess::{Board, ChessMove, MoveGen, Piece, EMPTY};

fn get_piece_weight(piece: Piece) -> i16 {
    // Return the estimated value of a piece
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 300,
        Piece::Bishop => 300,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 2500,
    }
}

#[derive(Eq)]
pub struct WeightedMove {
    pub chessmove: ChessMove,
    pub score: i16
}

impl Ord for WeightedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for WeightedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WeightedMove {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

pub fn order_moves(mut moves: MoveGen, board: Board, captures_only: bool) -> Vec<WeightedMove> {

    // Order_moves is responsible for taking in a movegen and returning a vector of moves that 
    // are ordered in some 'ideal' (heuristic) way.

    // Capture moves

    let mut moves_captures: Vec<WeightedMove> = Vec::new();

    let target_pieces = [Piece::Queen, Piece::King, Piece::Knight, Piece::Pawn, Piece::Rook];
    for target_piece in target_pieces{
        moves.set_iterator_mask(*board.color_combined(!board.side_to_move()) & board.pieces(target_piece));

        for capture_move in &mut moves {
            // Get value of piece we captured
            let captured_piece_wt = get_piece_weight(target_piece);

            // Get value of piece used in capture
            let own_piece_wt: i16;
            match board.piece_on(capture_move.get_source()) {
                Some(own_piece) => own_piece_wt = get_piece_weight(own_piece), // We should expect this, our piece has to start somewhere after all
                None => panic!("No piece on move origin"), // Panic for now
            }

            // Push both the weighted move struct to the vector
            moves_captures.push(WeightedMove { chessmove: capture_move, score: captured_piece_wt - own_piece_wt });
            // moves_captures.push(WeightedMove { chessmove: capture_move, score: 0});
        }
    }

    if !captures_only {
        // Sort captures (descending order)
        moves_captures.sort_by(|a, b| b.cmp(a));

        // Other moves (non-captures)

        let mut moves_other: Vec<WeightedMove> = Vec::new();
    
        moves.set_iterator_mask(!EMPTY);
        for other_move in &mut moves {
            moves_other.push(WeightedMove { chessmove: other_move, score: 0 });
        }

        // Sort other moves (descending order)
        // moves_other.sort_by(|a, b| b.cmp(a));

        moves_captures.append(&mut moves_other);
    }
    
    return moves_captures
}

