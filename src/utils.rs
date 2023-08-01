use std::ops::{AddAssign, Add};

use chess::Color;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Statistics {
    pub all_nodes: i32,
    pub searched_nodes: i32,
    pub caches_used: i32,
    pub time_ms: f32,
    pub depth_reached: u8,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EngineReturn {
    pub engine_move: String,
    pub engine_stats: Option<Statistics>,
}

pub(crate) fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b {
        a
    } else {
        b
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct CacheData {
    pub move_depth: i16,
    pub search_depth: i16,
    pub evaluation: Eval,
    pub move_type: HashtableResultType, // What type of move we have
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum HashtableResultType {
    RegularMove,
    PVMove,
    CutoffMove,
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