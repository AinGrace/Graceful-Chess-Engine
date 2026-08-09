use types::{MoveList, ScoredMoveList, chess_move::Move, role::Role};

// usage: [victim][attacker]
#[rustfmt::skip]
const MVV_LVA_TABLE: [[u8; Role::VARIANTS + 1]; Role::VARIANTS + 1] = [
    [10, 11, 12, 13, 14, 15, 0], // [P]     -> [K, Q, R, B, N, P, None]
    [20, 21, 22, 23, 24, 25, 0], // [N]     -> [K, Q, R, B, N, P, None]
    [30, 31, 32, 33, 34, 35, 0], // [B]     -> [K, Q, R, B, N, P, None]
    [40, 41, 42, 43, 44, 45, 0], // [R]     -> [K, Q, R, B, N, P, None]
    [50, 51, 52, 53, 54, 55, 0], // [Q]     -> [K, Q, R, B, N, P, None]
    [ 0,  0,  0,  0,  0,  0, 0], // [K]     -> [K, Q, R, B, N, P, None]
    [ 0,  0,  0,  0,  0,  0, 0], // [None]  -> [K, Q, R, B, N, P, None]
];

const NONE_IDX: usize = 6;
const TT_MOVE_SCORE: u8 = u8::MAX;

pub fn score_moves(moves: MoveList, tt_move: Option<Move>) -> ScoredMoveList {
    moves
        .iter()
        .map(|mv| {
            let score;
            if Some(*mv) == tt_move {
                score = TT_MOVE_SCORE;
            } else if let Some(captured_role) = mv.captured_role() {
                score = MVV_LVA_TABLE[captured_role.as_usize()][mv.role().as_usize()];
            } else {
                score = MVV_LVA_TABLE[NONE_IDX][mv.role().as_usize()];
            }

            (*mv, score)
        })
        .collect()
}

/// searches for the highest score move from scored_moves\[start_index\]..scored_moves.len()
/// and bubbles it up swapping the move at scored_moves\[stard_index\] with high scored move
pub fn bubble_high_scored_move(scored_moves: &mut ScoredMoveList, start_index: usize) {
    let mut high_score_move = start_index;
    for i in start_index + 1..scored_moves.len() {
        if scored_moves[i].1 > scored_moves[high_score_move].1 {
            high_score_move = i;
        }
    }

    scored_moves.swap(start_index, high_score_move);
}
