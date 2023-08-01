use std::sync::mpsc::{channel, self};
// use std::sync::mpsc::SyncSender::{Receiver, Sender};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

use chess::{Board, CacheTable};

use crate::utils::Eval;

pub struct Cache {
    pub cache: CacheTable<CacheData>,
    pub channel_tx: SyncSender<CacheEntry>,
    channel_rx: Receiver<CacheEntry>,
}

impl Cache {
    // This function should run in a seperate thread and constantly check for new data to load into the cache
    pub fn cache_manager_server(&mut self) {
        loop {
            let cache_rx = self.channel_rx.recv();
            match cache_rx {
                Ok(new_cache_entry) => self
                    .cache
                    .add(new_cache_entry.board.get_hash(), new_cache_entry.cachedata),
                Err(_) => {
                    println!("Exiting cache server");
                    break;
                } // No senders left break
            }
        }
    }

    pub fn cache_manager_get(&self, board_hash: u64) -> Option<CacheData>{
        self.cache.get(board_hash)
    }
}

impl Default for Cache {
    fn default() -> Self {
        // Set the buffer for 2000 messages, (hopefully we can handle that...)
        let (tx, rx): (SyncSender<CacheEntry>, Receiver<CacheEntry>) = sync_channel(2000);
        Cache {
            cache: CacheTable::new(
                67108864,
                CacheData {
                    move_depth: 0,
                    search_depth: 0,
                    evaluation: Eval { score: 0 },
                    move_type: HashtableResultType::RegularMove,
                },
            ),
            channel_tx: tx,
            channel_rx: rx,
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

#[cfg(test)]
mod tests {
    use super::CacheEntry;
    use crate::{
        managers::cache_manager::{Cache, CacheData, HashtableResultType},
        utils::Eval,
    };

    use std::sync::Arc;
    use std::thread;

    #[tokio::test]
    async fn test_cache_server() {
        // Declare cache table for transpositions

        let cache = Arc::new(Cache::default());
        // let mut cache = Arc<>;

        // tokio::spawn(async move {cache.cache_manager_server()});
        thread::spawn(|| {cache.cache_manager_server()});

        let cache_data = CacheData {
            move_depth: 1,
            search_depth: 2,
            evaluation: Eval { score: 3 },
            move_type: HashtableResultType::PVMove,
        };

        let cache_entry_to_send = CacheEntry {
            board: Default::default(),
            cachedata: cache_data.clone(),
        };
        
        cache.channel_tx.send(cache_entry_to_send).unwrap();

        let cache_retrieve = cache.cache_manager_get(cache_entry_to_send.board.get_hash()).unwrap();
        // let cache_retrieve = cache.cache.get(cache_entry_to_send.board.get_hash()).unwrap();
        
        assert_eq!(cache_retrieve, cache_data)

        // TODO complete test
        // Send a few CacheEntries down the tx pipe
        // Check wth a borrowed cache that they exist in the cache after a slight delay
    }
}
