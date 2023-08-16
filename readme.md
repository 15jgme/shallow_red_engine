# Shallow Red engine

[![Rust](https://github.com/15jgme/shallow_red_engine/actions/workflows/rust.yml/badge.svg)](https://github.com/15jgme/shallow_red_engine/actions/workflows/rust.yml)

This repo houses the core Shallow Red chess engine. Operation of this engine is very simple, it uses [jordanbray/chess](https://github.com/jordanbray/chess) for move generation, and general Chess classes. The engine takes in a board type and outputs a move type.

## Features 
### Implemented
- negamax
- alpha-beta pruning
- move ordering
  - PV moves 
  - cutoff moves
  - cached moves
  - Most Valuable Victim - Least Valuable Aggressor moves
- transposition tables
- iterative deepening
- quiecent search

### Not implemented
- opening book
- endgame deep searching

## Running the engine
```rust
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

```

## Playing the engine
The easiest way to play the engine is to check it out on Lichess @[ShallowRedBot](https://lichess.org/@/ShallowRedBot). You can also try the [UCI wrapper](https://github.com/15jgme/uci-shallow-red), or play with the [tauri GUI for Shallow Red](https://github.com/15jgme/shallow-red/releases).
