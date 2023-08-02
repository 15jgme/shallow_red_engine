// Houses the settings interface struct for searches

use std::time::SystemTime;

use chess::{Color, ChessMove};

use crate::managers::{stats_manager::StatisticsInputGrouping, cache_manager::CacheInputGrouping};

pub struct SearchParameters<'a>{
    pub depth: i16,
    pub depth_lim: i16,
    pub alpha: i16,
    pub beta: i16,
    pub color: Color,
    pub stats: StatisticsInputGrouping,
    pub cache: CacheInputGrouping,
    pub t_start: &'a SystemTime,
    pub first_search_move: Option<ChessMove>,
}