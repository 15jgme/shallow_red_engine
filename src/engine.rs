#![allow(dead_code)]
#![allow(unused_imports)]

use chess::{BitBoard, Board, BoardStatus, CacheTable, ChessMove, Color, MoveGen, Piece, EMPTY};
use std::time::Duration;
use std::time::SystemTime;

use crate::consts;
use crate::evaluation;
use crate::evaluation::evaluate_board;
use crate::ordering;

// use std::thread;
const DEPTH_LIM: i16 = 20;
const QUIESENT_LIM: i16 = 4;
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
    pub(crate) evaluation: Eval,
    pub(crate) move_type: HashtableResultType, // What type of move we have
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum HashtableResultType {
    RegularMove,
    PVMove,
    CutoffMove,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub(crate) struct Eval {
    pub(crate) score: i16, // Score is always (ALWAYS!) expressed as + is winning for White
}

impl Eval {
    pub(crate) fn for_colour(&self, colour: Color) -> i16 {
        // Returns the score for a colour where positive is more desriable for that colour
        match colour {
            Color::White => self.score,
            Color::Black => -self.score,
        }
    }
}

fn abs_eval_from_color(eval_rel: i16, color: Color) -> Eval {
    // Function provides a global eval struct from a local evaluation
    // specific to one colour, and the colour it is specific to.

    let eval_glob = match color {
        Color::White => eval_rel,  // + white
        Color::Black => -eval_rel, // Must be flipped for black
    };
    Eval { score: eval_glob }
}

fn flip_colour(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

fn find_best_move(
    board: Board,
    depth: i16,
    depth_lim: i16,
    mut alpha: i16,
    beta: i16,
    color_i: Color,
    stats_data: &mut Statistics,
    cache: &mut CacheTable<CacheData>,
) -> (Eval, ChessMove, [ChessMove; DEPTH_LIM as usize]) {
    // Copy alpha beta from parent

    if (depth >= depth_lim)
        || (board.status() == BoardStatus::Checkmate)
        || (board.status() == BoardStatus::Stalemate)
    {
        let mut _blank_move: ChessMove;
        let proposed_line: [ChessMove; DEPTH_LIM as usize] =
            [Default::default(); DEPTH_LIM as usize];

        // Note, issues with pruning, does weird things
        return (
            search_captures(&board, alpha, beta, 0, color_i, cache, depth_lim),
            Default::default(),
            proposed_line,
        );

        // return (
        //     evaluate_board(board),
        //     Default::default(),
        //     proposed_line,
        // );
    }

    // Generate moves
    let child_moves = MoveGen::new_legal(&board);
    // Get length of moves
    let num_moves = child_moves.len();

    if SEARCH_INFO {
        stats_data.all_nodes += num_moves as i32
    }

    let mut sorted_moves = ordering::order_moves(child_moves, board, cache, false, depth_lim); // sort all the moves

    // Initialize with least desirable evaluation
    let mut max_val = match color_i {
        Color::White => crate::consts::UNDESIRABLE_EVAL_WHITE,
        Color::Black => crate::consts::UNDESIRABLE_EVAL_BLACK,
    };

    let mut max_move = sorted_moves[0].chessmove.clone();
    let mut max_line: [ChessMove; DEPTH_LIM as usize] = [max_move; DEPTH_LIM as usize];

    for weighted_move in &mut sorted_moves {
        let mve = weighted_move.chessmove;

        let (node_evaluation, _best_move, proposed_line);

        match weighted_move.evaluation {
            Some(eval) => {
                // We've found this move in the current search no need to assess
                node_evaluation = eval;
                _best_move = Default::default();
                proposed_line = [Default::default(); DEPTH_LIM as usize];
                stats_data.caches_used += 1;
            }
            None => {
                (node_evaluation, _best_move, proposed_line) = find_best_move(
                    board.make_move_new(mve),
                    depth + 1,
                    depth_lim,
                    -beta,
                    -alpha,
                    flip_colour(color_i),
                    stats_data,
                    cache,
                );

                // Add move to hash
                cache.add(
                    board.make_move_new(mve).get_hash(),
                    CacheData {
                        move_depth: depth,
                        search_depth: depth_lim,
                        evaluation: node_evaluation,
                        move_type: HashtableResultType::RegularMove,
                    },
                );
            }
        }

        // Update stats
        if SEARCH_INFO {
            stats_data.searched_nodes += 1
        }

        // Replace with best move if we determine the move is the best for our current board side
        if node_evaluation.for_colour(color_i) > max_val.for_colour(color_i) {
            max_val = node_evaluation;
            max_move = mve;
            max_line = proposed_line;
            max_line[depth as usize] = max_move;
        }

        if DEBUG_MODE {
            println!("Move under consideration {}, number of possible moves {}, evaluation (for colour) {}, depth {}, colour {:#?}", mve, num_moves, node_evaluation.for_colour(color_i), depth, color_i)
        }

        alpha = max(alpha, node_evaluation.for_colour(board.side_to_move()));

        if alpha >= beta {
            // Alpha beta cutoff here

            // Record in cache that this is a cutoff move
            cache.add(
                board.make_move_new(mve).get_hash(),
                CacheData {
                    move_depth: depth,
                    search_depth: depth_lim,
                    evaluation: node_evaluation,
                    move_type: HashtableResultType::CutoffMove,
                },
            );

            break;
        }
    }

    // Overwrite the PV move in the hash
    cache.add(
        board.make_move_new(max_move).get_hash(),
        CacheData {
            move_depth: depth,
            search_depth: depth_lim,
            evaluation: max_val,
            move_type: HashtableResultType::PVMove,
        },
    );

    return (max_val, max_move, max_line);
}

fn search_captures(
    board: &Board,
    alpha_old: i16,
    beta: i16,
    depth: i16,
    color_i: Color,
    cache: &mut CacheTable<CacheData>,
    depth_lim: i16,
) -> Eval {
    let mut alpha = alpha_old;

    // Search through all terminal captures
    let stand_pat = evaluate_board(*board);
    if stand_pat.for_colour(color_i) >= beta || depth > QUIESENT_LIM {
        return abs_eval_from_color(beta, color_i);
    }

    alpha = max(alpha, stand_pat.for_colour(color_i));

    let capture_moves = MoveGen::new_legal(&board);
    let sorted_moves = ordering::order_moves(capture_moves, *board, cache, true, depth_lim); // sort all the moves

    for capture_move_score in sorted_moves {
        let capture_move = capture_move_score.chessmove;
        let score = search_captures(
            &board.make_move_new(capture_move),
            -beta,
            -alpha,
            depth + 1,
            flip_colour(color_i),
            cache,
            depth_lim,
        );

        if score.for_colour(color_i) >= beta {
            return abs_eval_from_color(beta, color_i);
        }

        alpha = max(alpha, score.for_colour(color_i));
    }

    return abs_eval_from_color(alpha, color_i);
}

pub async fn enter_engine(board: Board) -> ChessMove {
    println!("=============================================");
    println!("Balance of board {}", evaluate_board(board).score);

    let start_time = SystemTime::now();

    let color_i: Color = board.side_to_move();

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
            evaluation: Eval { score: 0 },
            move_type: HashtableResultType::RegularMove,
        },
    );

    let t_start = SystemTime::now(); // Initial time before running

    let mut terminal_depth: i16 = 1; // Starting depth

    let mut best_score: Eval = Eval { score: 0 };
    let mut best_mve: ChessMove = Default::default();
    let mut best_line: [ChessMove; DEPTH_LIM as usize] = Default::default();

    while t_start.elapsed().unwrap() < Duration::new(5, 0) && terminal_depth <= DEPTH_LIM {
        // Run until we hit the timelimit
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
            "Best move: {}, board score of best move (global): {}",
            best_mve, best_score.score
        );
    }

    println!(
        "Best move: {}, board score of best move: {}",
        best_mve, best_score.score
    );

    println!("Proposed line:");
    let mut i: i8 = 1;
    let mut is_white = color_i == Color::White;
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
