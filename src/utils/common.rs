use std::ops::{AddAssign, Add};
use chess::Color;
use crate::managers::stats_manager::{Statistics, StatisticsDepth};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EngineReturn {
    pub engine_move: String,
    pub engine_search_stats: Option<Statistics>,
    pub engine_depth_stats: Option<StatisticsDepth>
}

pub(crate) fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b {
        a
    } else {
        b
    }
}

pub(crate) fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b {
        b
    } else {
        a
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub struct Eval {
    pub score: i16, // Score is always (ALWAYS!) expressed as + is winning for White
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

impl Add for Eval {
    type Output = Eval;

    fn add(self, rhs: Eval) -> Self::Output {
        Eval {
            score: self.score + rhs.score,
        }
    }
}

impl AddAssign for Eval {
    fn add_assign(&mut self, rhs: Eval) {
        self.score += rhs.score
    }
}

pub(crate) fn abs_eval_from_color(eval_rel: i16, color: Color) -> Eval {
    // Function provides a global eval struct from a local evaluation
    // specific to one colour, and the colour it is specific to.

    let eval_glob = match color {
        Color::White => eval_rel,  // + white
        Color::Black => -eval_rel, // Must be flipped for black
    };
    Eval { score: eval_glob }
}

pub(crate) fn flip_colour(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}