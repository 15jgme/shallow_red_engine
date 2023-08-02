use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, RwLock,
};

use chess::{Board, CacheTable};

use crate::utils::common::Eval;

pub struct Cache {
    pub cache: CacheTable<CacheData>,
}

impl Cache {
    // // This function should run in a seperate thread and constantly check for new data to load into the cache
    // pub fn cache_manager_server(mut self, channel_rx: Receiver<CacheEntry>) {
    //     loop {
    //         let cache_rx = channel_rx.recv();
    //         match cache_rx {
    //             Ok(new_cache_entry) => self
    //                 .cache
    //                 .add(new_cache_entry.board.get_hash(), new_cache_entry.cachedata),
    //             Err(_) => {
    //                 println!("Exiting cache server");
    //                 break;
    //             } // No senders left break
    //         }
    //     }
    // }
    // This function should run in a seperate thread and constantly check for new data to load into the cache
    pub fn cache_manager_server(arc_cache: Arc<RwLock<Cache>>, channel_rx: Receiver<CacheEntry>) {
        let binding = arc_cache.clone();
        loop {
            let cache_rx = channel_rx.recv();
            match cache_rx {
                Ok(new_cache_entry) => {
                    // println!("Message received: {:#?}", new_cache_entry);
                    let mut cache = binding.write().unwrap();
                    cache
                        .cache
                        .add(new_cache_entry.board.get_hash(), new_cache_entry.cachedata)
                }
                Err(_) => {
                    println!("Exiting cache server");
                    break;
                } // No senders left break
            }
        }
    }

    pub fn cache_manager_get(&self, board_hash: u64) -> Option<CacheData> {
        self.cache.get(board_hash)
    }

    pub fn generate_channel() -> (Sender<CacheEntry>, Receiver<CacheEntry>) {
        let (tx, rx): (Sender<CacheEntry>, Receiver<CacheEntry>) = mpsc::channel();
        (tx, rx)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            cache: CacheTable::new(
                67108864,
                CacheData {
                    move_depth: 0,
                    search_depth: 0,
                    evaluation: Eval { score: 0 },
                    move_type: HashtableResultType::RegularMove,
                },
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct CacheData {
    pub move_depth: i16,
    pub search_depth: i16,
    pub evaluation: Eval,
    pub move_type: HashtableResultType, // What type of move we have
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum HashtableResultType {
    RegularMove,
    PVMove,
    CutoffMove,
}

// The engine uses this struct to ingest cache data from the engine
#[derive(Debug, Clone, Copy)]
pub struct CacheEntry {
    pub board: Board,
    pub cachedata: CacheData,
}

// Groups together the Cache pointer and the transmitter so that a section of the code
// Can interact with the cache
#[derive(Clone)]
pub struct CacheInputGrouping {
    pub cache_ref: Arc<RwLock<Cache>>,
    pub cache_tx: Sender<CacheEntry>,
}

#[cfg(test)]
mod tests {
    use super::CacheEntry;
    use crate::{
        managers::cache_manager::{Cache, CacheData, HashtableResultType},
        utils::common::Eval,
    };
    use std::thread;
    use std::{
        sync::{Arc, RwLock},
        time::Duration,
    };

    #[test]
    fn test_cache_server() {
        // Declare cache table for transpositions
        let cache_arc = Arc::new(RwLock::new(Cache::default()));
        let cache_arc_thread = cache_arc.clone();

        let (cache_tx, cache_rx) = Cache::generate_channel();

        let _cache_thread_hndl =
            thread::spawn(move || Cache::cache_manager_server(cache_arc_thread, cache_rx));

        let cache_data = CacheData {
            move_depth: 1,
            search_depth: 2,
            evaluation: Eval { score: 3 },
            move_type: HashtableResultType::PVMove,
        };

        let cache_entry_to_send = CacheEntry {
            board: Default::default(),
            cachedata: cache_data,
        };

        cache_tx.send(cache_entry_to_send).unwrap();

        thread::sleep(Duration::from_nanos(1));

        let cache_retrieve = cache_arc
            .read()
            .unwrap()
            .cache_manager_get(cache_entry_to_send.board.get_hash())
            .unwrap();

        // let _ = cache_thread_hndl.join();
        assert_eq!(cache_retrieve, cache_data);

        // TODO complete test
        // Send a few CacheEntries down the tx pipe
        // Check wth a borrowed cache that they exist in the cache after a slight delay
    }
}
