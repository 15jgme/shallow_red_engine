use crate::engine::Eval;

pub(crate) const UNDESIRABLE_EVAL_BLACK: Eval = Eval{score: i16::max_value() - 1};
pub(crate) const UNDESIRABLE_EVAL_WHITE: Eval = Eval{score: i16::min_value() + 1};