// Calling the enging can be done in only a few lines

use chess::Board;
use shallow_red_engine::engine::enter_engine;
use shallow_red_engine::utils::engine_interface::EngineSettings;

fn main() {
    // board with initial position
    let board = Board::default();

    // run the engine using board (in this case white will move)
    let (engine_move, _) = enter_engine(board, EngineSettings::default());

    // Print out the move we found
    println!(
        "The best move for white suggested by the engine is: {}",
        engine_move
    );

    // If you want to see more details about the engine, a second return of type EngineReturn is provided
    let (_engine_move, engine_data) = enter_engine(board, EngineSettings::default());
    println!(
        "The engine returned the following data: \n {:#?}",
        engine_data.unwrap()
    )
}
