use std::str::FromStr;

use chess::Board;
use shallow_red_engine::engine::enter_engine;

#[tokio::test]
async fn test_integrated_initial() {
    let board: Board = Board::default(); // Initial board
    let (eng_move, _) = enter_engine(board, false, None).await;
    assert!(board.legal(eng_move)); // Make sure the engine move is legal
}

#[tokio::test]
async fn test_integrated_endgame() {
    let board: Board = Board::from_str("3r4/8/3k4/8/8/3K4/8/8 b - - 0 1").unwrap();
    let (eng_move, _) = enter_engine(board, false, None).await;
    assert!(board.legal(eng_move)); // Make sure the engine move is legal
}