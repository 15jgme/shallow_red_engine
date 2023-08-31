// Houses the settings interface struct for searches

use std::time::{SystemTime, Duration};

use chess::{ChessMove, Color};


use crate::managers::cache_manager::CacheInputGrouping;
use crate::managers::stats_manager::Statistics;

use super::common::Eval;

pub struct SearchParameters<'a> {
    pub depth: u8,
    pub depth_lim: u8,
    pub extension: u8,
    pub alpha: i16,
    pub beta: i16,
    pub color: Color,
    pub cache: CacheInputGrouping,
    pub t_start: &'a SystemTime,
    pub t_lim: Duration,
    pub first_search_move: Option<ChessMove>,
    pub alternate_eval_fn: Option<fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) -> i16>
}

pub struct SearchOutput {
    pub node_eval: Eval,
    pub best_move: ChessMove,
    pub node_stats: Statistics, // Statistics as seen by OUR current node
}
