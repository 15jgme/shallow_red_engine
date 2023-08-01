#[cfg(test)]
mod tests {
    // use::super*;

    use std::{str::FromStr, time::SystemTime};

    use chess::{Board, CacheTable, ChessMove, Square};

    use shallow_red_engine::utils::{CacheData, Eval, HashtableResultType, Statistics};
    use shallow_red_engine::search::find_best_move;

    #[test]
    fn test_bug() {
        let fen = "r1k2b1r/ppp1nNpp/2p5/4P3/5Bb1/2N5/PPP2P1P/R4RK1 b - - 0 1";
        let board = Board::from_str(fen).expect("board should be valid");
        let mve = ChessMove::new(Square::H8, Square::G8, None);

        let mut run_stats = Statistics {
            all_nodes: 0,
            searched_nodes: 0,
            caches_used: 0,
            time_ms: 0.0,
            depth_reached: 1,
        };

        // Declare cache table for transpositions
        let mut cache: CacheTable<CacheData> = CacheTable::new(
            67108864,
            CacheData {
                move_depth: 0,
                search_depth: 0,
                evaluation: Eval { score: 0 },
                move_type: HashtableResultType::RegularMove,
            },
        );

        let t_start = SystemTime::now(); // Initial time before running

        let search_res = find_best_move(
            board,
            0,
            3,
            i16::MAX - 1,
            i16::MIN + 1,
            board.side_to_move(),
            &mut run_stats,
            &mut cache,
            &t_start,
            Some(mve),
        )
        .unwrap();
        println!("{:#?}", search_res.1.to_string());
        assert_ne!(search_res.1, ChessMove::new(Square::C3, Square::E4, None))
    }
}