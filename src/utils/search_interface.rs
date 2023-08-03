// Houses the settings interface struct for searches

use std::time::{SystemTime, Duration};

use chess::{ChessMove, Color};

use crate::consts;
use crate::managers::cache_manager::CacheInputGrouping;
use crate::managers::stats_manager::Statistics;

use super::common::Eval;

pub struct SearchParameters<'a> {
    pub depth: i16,
    pub depth_lim: i16,
    pub alpha: i16,
    pub beta: i16,
    pub color: Color,
    pub cache: CacheInputGrouping,
    pub t_start: &'a SystemTime,
    pub t_lim: Duration,
    pub first_search_move: Option<ChessMove>,
}

pub struct SearchOutput {
    pub node_eval: Eval,
    pub best_move: ChessMove,
    pub best_line: [ChessMove; consts::DEPTH_LIM as usize],
    pub node_stats: Statistics, // Statistics as seen by OUR current node
}
