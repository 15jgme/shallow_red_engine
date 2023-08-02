use std::time::Duration;

use crate::utils::common::Eval;

pub(crate) const UNDESIRABLE_EVAL_BLACK: Eval = Eval{score: i16::max_value() - 1};
pub(crate) const UNDESIRABLE_EVAL_WHITE: Eval = Eval{score: i16::min_value() + 1};
pub(crate) const DEPTH_LIM: i16 = 20;
pub(crate) const QUIESENT_LIM: i16 = 7;
pub(crate) const TIME_LIM: Duration = Duration::new(7, 0);
pub(crate) static DEBUG_MODE: bool = false;
pub(crate) static SEARCH_INFO: bool = true;
pub(crate) const MAX_DEPTH_TO_CHECK_TIME: i16 = 2;
pub(crate) const USE_CACHE: bool = true; // Flag to enable/disable move caching