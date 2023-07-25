use crate::engine::Eval;

pub(crate) const UNDESIRABLE_EVAL_BLACK: Eval = Eval{score: i16::max_value() - 1};
pub(crate) const UNDESIRABLE_EVAL_WHITE: Eval = Eval{score: i16::min_value() + 1};
pub(crate) const DEPTH_LIM: i16 = 20;
pub(crate) const QUIESENT_LIM: i16 = 4;
pub(crate) const TIME_LIM: u32 = 5000; // ms
pub(crate) static DEBUG_MODE: bool = false;
pub(crate) static SEARCH_INFO: bool = true;