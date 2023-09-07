// Houses the settings interface struct for running the engine

use std::{fmt::Debug, sync::mpsc::Receiver, time::Duration};

use crate::managers::cache_manager::CacheInputGrouping;

use super::common::Eval;

pub struct EngineSettings <EvalFunc>
where EvalFunc : Fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) -> i16 {
    pub cache_settings: Option<CacheInputGrouping>, // Settings to provide an external cache to the simulation
    pub time_limit: Duration, // Search will try to return ASAP after this limit
    pub stop_engine_rcv: Option<Receiver<bool>>, // Stop asap if this channel writes true
    pub verbose: bool,        // Engine will print extra data to stdout
    pub alternate_eval_func: Option<EvalFunc>,
} 

// impl<EvalFunc> EngineSettings <EvalFunc>
// where EvalFunc : Fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) -> i16 {
//     fn new() -> EngineSettings<EvalFunc> {
//         return ;
//     }
// }

pub fn new_settings(){
    
}

impl <EvalFunc> Debug for EngineSettings <EvalFunc>
where EvalFunc : Fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) -> i16  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineSettings")
            .field("time_limit", &self.time_limit)
            .field("stop_engine_rcv", &self.stop_engine_rcv)
            .field("verbose", &self.verbose)
            .finish()
    }
}

// Engine default
impl <EvalFunc> Default for EngineSettings <EvalFunc>
where EvalFunc : Fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) -> i16  {
    fn default() -> Self {
        // fn default_eval (a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, g: usize, h: usize, i: usize, j: usize, k: usize, l: usize) -> i16 {0}
        Self {
            cache_settings: None,               // Use internal Cache
            time_limit: Duration::from_secs(7), // Use a time limit of 7 sec
            stop_engine_rcv: None,              // Ignore the channel
            verbose: false, // Assume we're using a UCI interface so avoid stdout
            alternate_eval_func: None,
        }
    }
}
