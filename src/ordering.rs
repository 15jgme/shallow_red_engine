use std::cmp::Ordering;

// use itertools::Itertools;

use chess::{Board, ChessMove, MoveGen, Piece, EMPTY};

use crate::engine::CacheData;

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
    pub score: i16,
    pub evaluation: Option<i16>, // If we found an evaluation at the same depth search
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

pub(crate) fn order_moves(
    mut moves: MoveGen,
    board: Board,
    cache: &mut chess::CacheTable<CacheData>,
    captures_only: bool,
    search_depth: i16,
) -> Vec<WeightedMove> {
    // Order_moves is responsible for taking in a movegen and returning a vector of moves that
    // are ordered in some 'ideal' (heuristic) way.

    // Capture moves

    let mut moves_captures_cached: Vec<WeightedMove> = Vec::new();
    let mut moves_captures: Vec<WeightedMove> = Vec::new();

    let target_pieces = [
        Piece::Queen,
        Piece::King,
        Piece::Knight,
        Piece::Pawn,
        Piece::Rook,
    ];
    for target_piece in target_pieces {
        moves.set_iterator_mask(
            *board.color_combined(!board.side_to_move()) & board.pieces(target_piece),
        );

        for capture_move in &mut moves {
            // Get value of piece we captured
            let captured_piece_wt = get_piece_weight(target_piece);

            // Get value of piece used in capture
            let own_piece_wt: i16;
            match board.piece_on(capture_move.get_source()) {
                Some(own_piece) => own_piece_wt = get_piece_weight(own_piece), // We should expect this, our piece has to start somewhere after all
                None => panic!("No piece on move origin"),                     // Panic for now
            }
            // moves_captures.push(WeightedMove { chessmove: capture_move, score: 0});

            // Check if this move is in our cache
            match cache.get(board.make_move_new(capture_move).get_hash()) {
                Some(cache_result) => {
                    // Move found in cache
                    // Check if we found it at the current search depth to see if evaluation is valid
                    let evaluation_valid = cache_result.search_depth == search_depth;

                    let evaluation: Option<i16> = if evaluation_valid {
                        Some(cache_result.evaluation)
                    } else {
                        None
                    };

                    // Push the weighted move struct to the vector
                    moves_captures.push(WeightedMove {
                        chessmove: capture_move,
                        score: cache_result.evaluation,
                        evaluation: evaluation,
                    });
                }
                None => {
                    // Move not found in cache
                    // Push the weighted move struct to the vector
                    moves_captures.push(WeightedMove {
                        chessmove: capture_move,
                        score: captured_piece_wt - own_piece_wt,
                        evaluation: None,
                    });
                }
            }
        }
    }

    // Sort captures (descending order)
    moves_captures_cached.sort_by(|a, b| b.cmp(a));
    moves_captures.sort_by(|a, b| b.cmp(a));

    if !captures_only {
        // Other moves (non-captures)

        let mut moves_other_cached: Vec<WeightedMove> = Vec::new();
        let mut moves_other: Vec<WeightedMove> = Vec::new();

        moves.set_iterator_mask(!EMPTY);
        for other_move in &mut moves {
            match cache.get(board.make_move_new(other_move).get_hash()) {
                Some(cache_result) => {
                    // Move found in cache
                    // Check if we found it at the current search depth to see if evaluation is valid
                    let evaluation_valid = cache_result.search_depth == search_depth;

                    let evaluation: Option<i16> = if evaluation_valid {
                        Some(cache_result.evaluation)
                    } else {
                        None
                    };

                    // Push the weighted move struct to the vector
                    moves_other_cached.push(WeightedMove {
                        chessmove: other_move,
                        score: cache_result.evaluation,
                        evaluation: evaluation,
                    });
                }
                None => {
                    // Move not found in cache
                    // Push the weighted move struct to the vector
                    moves_other.push(WeightedMove {
                        chessmove: other_move,
                        score: 0,
                        evaluation: None,
                    });
                }
            }
        }

        // Sort other moves (descending order)
        moves_other_cached.sort_by(|a, b| b.cmp(a));

        // Order is as follows, cached capture moves > capture moves > cached non-captures > non-captures
        moves_captures_cached.append(&mut moves_captures);
        moves_captures_cached.append(&mut moves_other_cached);
        moves_captures_cached.append(&mut moves_other);
    }

    return moves_captures_cached;
}
