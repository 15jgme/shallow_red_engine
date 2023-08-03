use std::{str::FromStr, sync::{Arc, RwLock}, thread};

use chess::Board;
use shallow_red_engine::{engine::enter_engine, utils::engine_interface::EngineSettings, managers::cache_manager::{Cache, CacheInputGrouping}};

#[tokio::test]
async fn test_integrated_initial() {
    let board: Board = Board::default(); // Initial board
    let (eng_move, _) = enter_engine(board, EngineSettings::default()).await;
    assert!(board.legal(eng_move)); // Make sure the engine move is legal
}

#[tokio::test]
async fn test_integrated_endgame() {
    let board: Board = Board::from_str("3r4/8/3k4/8/8/3K4/8/8 b - - 0 1").unwrap();
    let (eng_move, _) = enter_engine(board, EngineSettings::default()).await;
    assert!(board.legal(eng_move)); // Make sure the engine move is legal
}

#[tokio::test]
    async fn test_external_cache() {
        // Declare cache table for transpositions
        let cache_arc = Arc::new(RwLock::new(Cache::default()));
        let cache_arc_thread = cache_arc.clone();

        let (cache_tx, cache_rx) = Cache::generate_channel();

        let _cache_thread_hndl =
            thread::spawn(move || Cache::cache_manager_server(cache_arc_thread, cache_rx));

        let mut settings = EngineSettings::default();
        settings.cache_settings = Some(CacheInputGrouping{ cache_ref: cache_arc, cache_tx });

        let board: Board = Board::from_str("3r4/8/3k4/8/8/3K4/8/8 b - - 0 1").unwrap();
        enter_engine(board, settings).await;
    }