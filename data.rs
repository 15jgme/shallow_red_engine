// File to store evaluation preference data

use std::collections::HashMap;
use chess::{Square};


pub fn foo (sq: Square) {
    let mut weights = HashMap::new();
    weights.insert(sq, -1);
}

