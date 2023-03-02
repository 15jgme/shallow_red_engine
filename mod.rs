use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen, Piece, EMPTY};
use std::f32::INFINITY;
mod data;
// use std::thread;
const DEPTH_LIM: i8 = 8;
static DEBUG_MODE: bool = false;
static SEARCH_INFO: bool = true;
// static MULTI_THREAD: bool = true;

struct Statistics {
    all_nodes: i32,
    searched_nodes: i32,
}

fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b {
        a
    } else {
        b
    }
}

fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b {
        a
    } else {
        b
    }
}

fn find_best_move(
    board: Board,
    depth: i8,
    mut alpha: i16,
    mut beta: i16,
    maximize: bool,
    stats_data: &mut Statistics,
) -> (i16, ChessMove, [ChessMove; DEPTH_LIM as usize]) {
    // Copy alpha beta from parent

    if (depth >= DEPTH_LIM)
        || (board.status() == BoardStatus::Checkmate)
        || (board.status() == BoardStatus::Stalemate)
    {
        let mut _blank_move: ChessMove;
        let proposed_line: [ChessMove; DEPTH_LIM as usize] =
            [Default::default(); DEPTH_LIM as usize];
        return (evaluate_board(board), Default::default(), proposed_line);
    }

    if maximize
    // Own player, try to maximize score
    {
        if board.side_to_move() != Color::White {
            panic!("Maximizing with black to move!")
        }

        let mut max_val = -INFINITY as i16;
        let mut max_move = Default::default();
        let mut max_line: [ChessMove; DEPTH_LIM as usize] =
            [Default::default(); DEPTH_LIM as usize];

        // Generate moves
        let mut child_moves = MoveGen::new_legal(&board);
        // Get length of moves
        let num_moves = child_moves.len();
        let targets = board.color_combined(!board.side_to_move());
        let targets = [*targets, !EMPTY];

        let mut continue_search: bool = true;

        if SEARCH_INFO {stats_data.all_nodes += num_moves as i32}

        for trg in targets {
            child_moves.set_iterator_mask(trg); // Set target mask

            if continue_search {
                for mve in &mut child_moves {
                    let (mve_score, _best_move, proposed_line) = find_best_move(
                        board.make_move_new(mve),
                        depth + 1,
                        alpha,
                        beta,
                        false,
                        stats_data,
                    );

                    // Update stats
                    if SEARCH_INFO {
                        stats_data.searched_nodes += 1
                    }

                    if mve_score > max_val {
                        max_val = mve_score;
                        max_move = mve;
                        max_line = proposed_line;
                        max_line[depth as usize] = max_move;
                    }
                    
                    if DEBUG_MODE {
                        println!("Move under consideration {}, number of possible moves {}, resulting score {}, depth {}, maximizing", mve, num_moves, mve_score, depth)
                    }
                    if beta <= alpha {
                        continue_search = false;
                        break;
                    }

                    alpha = max(alpha, mve_score);
                }
            }
        }

        return (max_val, max_move, max_line);
    } else {
        if board.side_to_move() != Color::Black {
            panic!("Minimizing with white to move!")
        }

        // Opponent, try to minimize score
        let mut min_val = INFINITY as i16;
        let mut min_move = Default::default();
        let mut min_line: [ChessMove; DEPTH_LIM as usize] =
            [Default::default(); DEPTH_LIM as usize];

        // Generate moves
        let mut child_moves = MoveGen::new_legal(&board);
        // Get length of moves
        let num_moves = child_moves.len();
        let targets = board.color_combined(!board.side_to_move());
        let targets = [*targets, !EMPTY];

        let mut continue_search: bool = true;

        if SEARCH_INFO {stats_data.all_nodes += num_moves as i32}

        for trg in targets {
            child_moves.set_iterator_mask(trg); // Set target mask

            if continue_search {
                for mve in &mut child_moves {
                    let (mve_score, _best_move, proposed_line) = find_best_move(
                        board.make_move_new(mve),
                        depth + 1,
                        alpha,
                        beta,
                        true,
                        stats_data,
                    );
                    // Update stats
                    if SEARCH_INFO {
                        stats_data.searched_nodes += 1
                    }

                    if mve_score < min_val {
                        min_val = mve_score;
                        min_move = mve;
                        min_line = proposed_line;
                        min_line[depth as usize] = min_move;
                    }
                    
                    if DEBUG_MODE {
                        println!("Move under consideration {}, number of possible moves {}, resulting score {}, depth {}, minimizing", mve, num_moves, mve_score, depth)
                    }
                    if beta <= alpha {
                        continue_search = false;
                        break;
                    }

                    beta = min(beta, mve_score);
                }
            }
        }

        return (min_val, min_move, min_line);
    }
}

fn evaluate_board(board: Board) -> i16 {
    // Returns the current score on the board where white winning is positive and black winning is negative

    match board.status() {
        BoardStatus::Checkmate => {
            // We are always in checkmate with the current side to move
            // Since checkmate ends the game, we only need to asses it once
            // Since we assess after a move, it is safe to check at the child node level
            match board.side_to_move() {
                Color::White => return -INFINITY as i16,
                Color::Black => return INFINITY as i16,
            }
        }
        BoardStatus::Stalemate => {
            return 0; // Stalemate is a draw game
        }
        BoardStatus::Ongoing => {
            // List of values
            let v_pawn: i16 = 100;
            let v_knight: i16 = 300;
            let v_bishop: i16 = 300;
            let v_rook: i16 = 500;
            let v_queen: i16 = 900;

            let v_king: i16 = 2500; // Temporary, ensure that the king is super valuable

            let mut score: i16 = 0;

            for p in board.pieces(Piece::Pawn) & board.color_combined(Color::Black){
                p
            }

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

            return score;
        }
    }
}

pub(crate) fn enter_engine(board: Board) {
    println!("Balance of board {}", evaluate_board(board));

    let maximize: bool = board.side_to_move() == Color::White;

    let mut run_stats = Statistics {
        all_nodes: 0,
        searched_nodes: 0,
    };

    let (best_score, best_mve, best_line) =
        find_best_move(board, 0, -INFINITY as i16, INFINITY as i16, maximize, &mut run_stats);
    println!(
        "Best move: {}, board score of best move: {}",
        best_mve, best_score
    );

    println!("Proposed line:");
    let mut i: i8 = 1;
    let mut is_white = maximize;
    for mve in best_line {
        if is_white {
            println!("White, Move {}: {}", i, mve);
        } else {
            println!("Black, Move {}: {}", i, mve);
        }
        
        is_white = !is_white;
        i += 1;
    }

    let percent_reduction: f32 = (1.0 - (run_stats.searched_nodes as f32)/(run_stats.all_nodes as f32)) * 100.0;
    if SEARCH_INFO {println!("Search stats. \n All nodes in problem: {}\n Nodes visited {}, reduction {}%", run_stats.all_nodes, run_stats.searched_nodes, percent_reduction)}
    
}
