#![allow(dead_code)]
#![allow(unused_imports)]

use std::time::Duration;
use std::time::SystemTime;

// use std::hash::Hash;
use chess::{BitBoard, Board, BoardStatus, CacheTable, ChessMove, Color, MoveGen, Piece, EMPTY};
// use itertools::Itertools;

use crate::ordering;

// use std::thread;
const DEPTH_LIM: i16 = 20;
// const QUIESENT_LIM: i16 = 4;
const TIME_LIM: u32 = 5000; // ms
static DEBUG_MODE: bool = false;
static SEARCH_INFO: bool = true;
// static MULTI_THREAD: bool = true;

struct Statistics {
    all_nodes: i32,
    searched_nodes: i32,
    caches_used: i32,
    time_ms: f32,
    depth_reached: u8,
}

fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b {
        a
    } else {
        b
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct CacheData {
    pub(crate) move_depth: i16,
    pub(crate) search_depth: i16,
    pub(crate) evaluation: i16,
}

fn find_best_move(
    board: Board,
    depth: i16,
    depth_lim: i16,
    mut alpha: i16,
    beta: i16,
    color_i: i8,
    stats_data: &mut Statistics,
    cache: &mut CacheTable<CacheData>,
) -> (i16, ChessMove, [ChessMove; DEPTH_LIM as usize]) {
    // Copy alpha beta from parent

    if (depth >= depth_lim)
        || (board.status() == BoardStatus::Checkmate)
        || (board.status() == BoardStatus::Stalemate)
    {
        let mut _blank_move: ChessMove;
        let proposed_line: [ChessMove; DEPTH_LIM as usize] =
            [Default::default(); DEPTH_LIM as usize];
        // Note, issues with pruning, does weird things
        // return (
        //     (color_i as i16) * search_captures(&board, alpha, beta, 0, color_i),
        //     Default::default(),
        //     proposed_line,
        // );

        return (
            (color_i as i16) * evaluate_board(board),
            Default::default(),
            proposed_line,
        );
    }

    let mut max_val = i16::min_value() + 1;
    let mut max_move = Default::default();
    let mut max_line: [ChessMove; DEPTH_LIM as usize] = [Default::default(); DEPTH_LIM as usize];

    // Generate moves
    let child_moves = MoveGen::new_legal(&board);
    // Get length of moves
    let num_moves = child_moves.len();

    if SEARCH_INFO {
        stats_data.all_nodes += num_moves as i32
    }

    let mut sorted_moves = ordering::order_moves(child_moves, board, cache, false, depth_lim); // sort all the moves

    for weighted_move in &mut sorted_moves {
        let mve = weighted_move.chessmove;

        let (negative_value, _best_move, proposed_line);

        match weighted_move.evaluation {
            Some(eval) => {
                // We've found this move in the current search no need to assess
                negative_value = -eval;
                _best_move = Default::default();
                proposed_line = [Default::default(); DEPTH_LIM as usize];
                stats_data.caches_used += 1;
            }
            None => {
                (negative_value, _best_move, proposed_line) = find_best_move(
                    board.make_move_new(mve),
                    depth + 1,
                    depth_lim,
                    -beta,
                    -alpha,
                    -color_i,
                    stats_data,
                    cache,
                );

                // Add move to hash
                cache.add(
                    board.make_move_new(mve).get_hash(),
                    CacheData {
                        move_depth: depth,
                        search_depth: depth_lim,
                        evaluation: -negative_value,
                    },
                );
            }
        }

        let value = -negative_value;

        // Update stats
        if SEARCH_INFO {
            stats_data.searched_nodes += 1
        }

        if value > max_val {
            max_val = value;
            max_move = mve;
            max_line = proposed_line;
            max_line[depth as usize] = max_move;
        }

        if DEBUG_MODE {
            println!("Move under consideration {}, number of possible moves {}, resulting score {}, depth {}, maximizing", mve, num_moves, -value, depth)
        }

        alpha = max(alpha, value);

        if alpha >= beta {
            break;
        }
    }

    return (max_val, max_move, max_line);
}

fn search_captures(
    board: &Board,
    alpha_old: i16,
    beta: i16,
    depth: i16,
    color_i: i8,
    cache: &mut CacheTable<CacheData>,
    depth_lim: i16,
) -> i16 {
    let mut alpha = alpha_old;

    // Search through all terminal captures
    let colour_at_depth: i16 = if depth == 0 { 1 } else { color_i as i16 };
    let stand_pat = colour_at_depth * evaluate_board(*board); // sign doesnt really matter still fucked up
    if stand_pat >= beta {
        return beta;
    }

    alpha = max(alpha, stand_pat);

    let capture_moves = MoveGen::new_legal(&board);
    let sorted_moves = ordering::order_moves(capture_moves, *board, cache, true, depth_lim); // sort all the moves

    for capture_move_score in sorted_moves {
        let capture_move = capture_move_score.chessmove;
        let score = search_captures(
            &board.make_move_new(capture_move),
            -beta,
            -alpha,
            depth + 1,
            -color_i,
            cache,
            depth_lim,
        );

        if stand_pat >= beta {
            return beta;
        }

        alpha = max(alpha, score);
    }

    return alpha;
}

fn evaluate_board(board: Board) -> i16 {
    // Returns the current score on the board where white winning is positive and black winning is negative

    match board.status() {
        BoardStatus::Checkmate => {
            // We are always in checkmate with the current side to move
            // Since checkmate ends the game, we only need to asses it once
            // Since we assess after a move, it is safe to check at the child node level
            match board.side_to_move() {
                Color::White => return i16::min_value() + 1,
                Color::Black => return i16::max_value() - 1,
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

            // for p in board.pieces(Piece::Pawn) & board.color_combined(Color::Black) {
            //     p
            // }

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

pub fn enter_engine(board: Board) -> ChessMove {
    println!("=============================================");
    println!("Balance of board {}", evaluate_board(board));

    let start_time = SystemTime::now();

    let color_i: i8 = if board.side_to_move() == Color::White {
        1
    } else {
        -1
    };
    // The color expressed as an integer, where white == 1 and black == -1

    let mut run_stats = Statistics {
        all_nodes: 0,
        searched_nodes: 0,
        caches_used: 0,
        time_ms: 0.0,
        depth_reached: 1,
    };

    // Declare cache table for transpositions
    let mut cache: CacheTable<CacheData> = CacheTable::new(
        65536,
        CacheData {
            move_depth: 0,
            search_depth: 0,
            evaluation: 0,
        },
    );

    let t_start = SystemTime::now(); // Initial time before running

    let mut terminal_depth: i16 = 1; // Starting depth

    let mut best_score: i16 = 0;
    let mut best_mve: ChessMove = Default::default();
    let mut best_line: [ChessMove; DEPTH_LIM as usize] = Default::default();

    while t_start.elapsed().unwrap() < Duration::new(5, 0) && terminal_depth <= DEPTH_LIM{ // Run until we hit the timelimit
        println!("Current depth {}", terminal_depth);
        (best_score, best_mve, best_line) = find_best_move(
            board.clone(),
            0,
            terminal_depth,
            i16::min_value() + 1,
            i16::max_value() - 1,
            color_i,
            &mut run_stats,
            &mut cache,
        );

        // Go farther each iteration
        terminal_depth += 1;

        println!(
            "Best move: {}, board score of best move: {}",
            best_mve, best_score
        );
    }
    



    println!(
        "Best move: {}, board score of best move: {}",
        best_mve, best_score
    );

    println!("Proposed line:");
    let mut i: i8 = 1;
    let mut is_white = color_i == 1;
    for mve in best_line {
        if is_white {
            println!("White, Move {}: {}", i, mve);
        } else {
            println!("Black, Move {}: {}", i, mve);
        }

        is_white = !is_white;
        i += 1;
    }

    let percent_reduction: f32 =
        (1.0 - (run_stats.searched_nodes as f32) / (run_stats.all_nodes as f32)) * 100.0;

    // get final time
    let end_time = SystemTime::now();
    match end_time.duration_since(start_time) {
        Ok(duration) => run_stats.time_ms = duration.as_millis() as f32,
        Err(_) => {}
    }

    if SEARCH_INFO {
        println!(
            "Search stats. \n All nodes in problem: {}\n Nodes visited {}, reduction {}%, times used cache {}, time elapsed (ms) {}",
            run_stats.all_nodes, run_stats.searched_nodes, percent_reduction, run_stats.caches_used, run_stats.time_ms,
        )
    }

    return best_mve;
}
