#[macro_export]
macro_rules! check_solve {
    ( $x:expr, $y:expr) => {
        {
            let board = Board::from_str($x).unwrap();
            let settings = EngineSettings {time_limit: Duration::from_secs(10), ..Default::default()};
            let (eng_move, _) = enter_engine(board, settings);
            assert_eq!(eng_move.to_string(), $y);
        }
    };
}

#[cfg(test)]
mod tests{
    // Test checkmate patter positions from Lichess 
    // https://lichess.org/practice/checkmates/checkmate-patterns-iii/
    // These tests cases are taken from flounderbot and restructured for shallow-red. Thanks Flounder!

    use std::{str::FromStr, time::Duration};

    use chess::Board;
    use shallow_red_engine::{engine::enter_engine, utils::engine_interface::EngineSettings};

    #[test]
    #[serial_test::serial]
    fn opera_mate_1() {
        check_solve!("4k3/5p2/8/6B1/8/8/8/3R2K1 w - - 0 1", "d1d8")
    }

    #[test]
    #[serial_test::serial]
    fn opera_mate_2() {
        check_solve!("rn1r2k1/ppp2ppp/3q1n2/4b1B1/4P1b1/1BP1Q3/PP3PPP/RN2K1NR b KQ - 0 1", "d6d1")
    }

    #[test]
    #[serial_test::serial]
    fn opera_mate_3() {
        check_solve!("rn3rk1/p5pp/2p5/3Ppb2/2q5/1Q6/PPPB2PP/R3K1NR b KQ - 0 1", "c4f1")
    }

    #[test]
    #[serial_test::serial]
    fn anderssens_mate_1() {
        check_solve!("6k1/6P1/5K1R/8/8/8/8/8 w - - 0 1", "h6h8")
    }

    #[test]
    #[serial_test::serial]
    fn anderssens_mate_2() {
        check_solve!("1k2r3/pP3pp1/8/3P1B1p/5q2/N1P2b2/PP3Pp1/R5K1 b - - 0 1", "f4h4")
    }

    // Not optimal, should fix
    // #[test]
    // #[serial_test::serial]
    // fn anderssens_mate_3() {
    //     check_solve!("2r1nrk1/p4p1p/1p2p1pQ/nPqbRN2/8/P2B4/1BP2PPP/3R2K1 w - - 0 1", "f5e7")
    // }

    #[test]
    #[serial_test::serial]
    fn dovetail_mate_1() {
        check_solve!("1r6/pk6/4Q3/3P4/8/8/8/6K1 w - - 0 1", "e6c6")
    }

    #[test]
    #[serial_test::serial]
    fn dovetail_mate_2() {
        check_solve!("r1b1q1r1/ppp3kp/1bnp4/4p1B1/3PP3/2P2Q2/PP3PPP/RN3RK1 w - - 0 1", "f3f6")
    }

    // Not optimal, should fix
    #[test]
    #[serial_test::serial]
    fn dovetail_mate_3() {
        check_solve!("6k1/1p1b3p/2pp2p1/p7/2Pb2Pq/1P1PpK2/P1N3RP/1RQ5 b - - 0 1", "d7g4")
    }

    #[test]
    #[serial_test::serial]
    fn dovetail_mate_4() {
        check_solve!("rR6/5k2/2p3q1/4Qpb1/2PB1Pb1/4P3/r5R1/6K1 w - - 0 1", "e5e8")
    }

    #[test]
    #[serial_test::serial]
    fn cozios_mate_1() {
        check_solve!("8/8/1Q6/8/6pk/5q2/8/6K1 w - - 0 1", "b6h6")
    }

    #[test]
    #[serial_test::serial]
    fn swallows_tail_mate_1() {
        check_solve!("3r1r2/4k3/R7/3Q4/8/8/8/6K1 w - - 0 1", "d5e6")
    }

    #[test]
    #[serial_test::serial]
    fn swallows_tail_mate_2() {
        check_solve!("8/8/2P5/3K1k2/2R3p1/2q5/8/8 b - - 0 1", "c3e5")
    }

    #[test]
    #[serial_test::serial]
    fn epaulette_mate_1() {
        check_solve!("3rkr2/8/5Q2/8/8/8/8/6K1 w - - 0 1", "f6e6")
    }

    #[test]
    #[serial_test::serial]
    fn epaulette_mate_2() {
        check_solve!("1k1r4/pp1q1B1p/3bQp2/2p2r2/P6P/2BnP3/1P6/5RKR b - - 0 1", "d8g8")
    }

    #[test]
    #[serial_test::serial]
    fn epaulette_mate_3() {
        check_solve!("5r2/pp3k2/5r2/q1p2Q2/3P4/6R1/PPP2PP1/1K6 w - - 0 1", "f5d7")
    }

    #[test]
    #[serial_test::serial]
    fn pawn_mate_1() {
        check_solve!("8/7R/1pkp4/2p5/1PP5/8/8/6K1 w - - 0 1", "b4b5")
    }

    #[test]
    #[serial_test::serial]
    fn pawn_mate_2() {
        check_solve!("r1b3nr/ppp3qp/1bnpk3/4p1BQ/3PP3/2P5/PP3PPP/RN3RK1 w - - 0 11", "h5e8")
    }
}