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

  use chess::{Board, ChessMove};
  use shallow_red_engine::enter_engine

  // board with initial position
  let board = Board::default();

  // run the engine using board (in this case white will move)
  let engine_move: ChessMove = enter_engine(board).await;

  // Print out the move we found
  println!("The best move for white suggested by the engine is: {}", engine_move.to_string());
```

## Playing the engine
The easiest way to test out the engine is to download the latest build for [Shallow Red](https://github.com/15jgme/shallow-red/releases). That is a project that provides a UI for this engine using Tauri (and can also be easily modified to run any engine with an input of a Board type and output of a ChessMove type)
