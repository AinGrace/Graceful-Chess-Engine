use types::{
    MoveList,
    bitboard::{
        self, Bitboard, ToBitboard,
        masks::{RANK_1, RANK_2, RANK_7, RANK_8},
    },
    castlings::Castlings,
    chess_move::Move,
    color::Color,
    direction::Direction,
    piece::Piece,
    role::Role,
    square::Square,
};

use crate::{board::Board, move_gen::pin_info::PinInfo, position::Position};

pub fn gen_legal_moves_for_v2<const CAPTURES_ONLY: bool>(pos: &Position, color: Color) -> MoveList {
    let mut moves = MoveList::new();
    let mut on_move = |mv| unsafe { moves.push_unchecked(mv) };

    unsafe {
        match color {
            Color::White => gen_legal_moves_white::<CAPTURES_ONLY>(pos, &mut on_move),
            Color::Black => gen_legal_moves_black::<CAPTURES_ONLY>(pos, &mut on_move),
        }
    }
    moves
}

unsafe fn gen_legal_moves_white<const CAPTURES_ONLY: bool>(
    pos: &Position,
    on_move: &mut impl FnMut(Move),
) {
    let board = pos.board();
    let king_checkers = pos.checkers_to(Color::White);
    let king_sqr = board.the_king(Color::White);
    let ep_sqr = pos.ep_square();

    let pin_info = PinInfo::compute(pos, Color::White);

    gen_white_king_moves::<CAPTURES_ONLY>(board, king_sqr, on_move);

    if king_checkers.popcnt() == 2 {
        return;
    }

    if king_checkers.popcnt() != 0 {
        let checker = unsafe { king_checkers.first_square_unchecked() };
        gen_white_evasion_moves::<CAPTURES_ONLY>(
            board, king_sqr, ep_sqr, checker, &pin_info, on_move,
        );
        return;
    }

    gen_white_moves::<CAPTURES_ONLY>(board, &pin_info, on_move);
    gen_white_castling_moves::<CAPTURES_ONLY>(board, king_sqr, *pos.castling_rights(), on_move);
    gen_white_ep_moves(pos, king_sqr, on_move);
}

fn gen_white_ep_moves(pos: &Position, king_sqr: Square, on_move: &mut impl FnMut(Move)) {
    let Some(ep) = pos.ep_square() else {
        return;
    };

    let board = pos.board();
    let occupied = board.occupied();

    let pawns = board.w_pawns();

    let enemy_bishops = board.b_bishops();
    let enemy_rooks = board.b_rooks();
    let enemy_queens = board.b_queens();

    let enemies = enemy_bishops | enemy_rooks | enemy_queens;

    let enemy_pawn = Piece::BPawn;

    pawns.for_each(|from| {
        if lookup::pawn_attacks(Color::White, from) & (1 << ep.as_u32()) == 0 {
            return;
        }

        let captured = Square::of(ep.file(), from.rank());

        if board.peek(captured) != Some(enemy_pawn) {
            return;
        }

        let legal = if enemies.empty() {
            true
        } else {
            let occupied = occupied
                .clear_square(from)
                .clear_square(captured)
                .set_square(ep);

            let opened_enemy_rook =
                lookup::rook_attacks(king_sqr, occupied.as_u64()).to_bb() & enemy_rooks;

            let opened_enemy_bishop =
                lookup::bishop_attacks(king_sqr, occupied.as_u64()).to_bb() & enemy_bishops;

            let opened_enemy_queen =
                lookup::queen_attacks(king_sqr, occupied.as_u64()).to_bb() & enemy_queens;

            (opened_enemy_rook | opened_enemy_bishop | opened_enemy_queen).empty()
        };

        if legal {
            on_move(Move::en_passant(from, ep));
        }
    });
}

fn gen_white_castling_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    king_sqr: Square,
    castling_rights: Castlings,
    on_move: &mut impl FnMut(Move),
) {
    if CAPTURES_ONLY {
        return;
    }

    let occupied = board.occupied();

    if castling_rights.w_short() {
        let f = Square::F1;
        let g = Square::G1;

        let path = f.to_bb().set_square(g);

        if !occupied.intersects(path)
            && !board.is_square_attacked_by(f, Color::Black)
            && !board.is_square_attacked_by(g, Color::Black)
        {
            on_move(Move::king_castle(king_sqr, g));
        }
    }

    if castling_rights.w_long() {
        let d = Square::D1;
        let c = Square::C1;
        let b = Square::B1;

        let clear = d.to_bb().set_square(c).set_square(b);

        if !occupied.intersects(clear)
            && !board.is_square_attacked_by(d, Color::Black)
            && !board.is_square_attacked_by(c, Color::Black)
        {
            on_move(Move::queen_castle(king_sqr, c));
        }
    }
}

fn gen_white_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    pin_info: &PinInfo,
    on_move: &mut impl FnMut(Move),
) {
    let pinned = pin_info.pinned();
    let b_pins = pin_info.b_pins();
    let r_pins = pin_info.r_pins();

    let pawns = board.w_pawns();
    let knights = board.w_knights();
    let bishops = board.w_bishops();
    let rooks = board.w_rooks();
    let queens = board.w_queens();

    let occupied_e = board.blacks();
    let occupied_m = board.whites();
    let occupied_all = board.occupied();

    let pinned_m = occupied_m & pinned;
    let unpinned_m = occupied_m & !pinned;

    let pawns_attk = pawns & !r_pins; // rook-pinned pawns can not capture
    let pawns_push = pawns & !b_pins; // diagonally pinned pawns can not push forward

    let pawns_left_captures = {
        let left_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::NorthWest);
        let left_pinned = (pawns_attk & b_pins).shift_dir(Direction::NorthWest) & b_pins;
        let pawns_left_all = left_pinned | left_unpinned;
        pawns_left_all & occupied_e
    };

    let pawns_right_captures = {
        let right_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::NorthEast);
        let right_pinned = (pawns_attk & b_pins).shift_dir(Direction::NorthEast) & b_pins;
        let pawns_right_all = right_pinned | right_unpinned;
        pawns_right_all & occupied_e
    };

    let pawns_forward = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::North) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::North) & r_pins & !occupied_all;
        forward_unpinned | forward_pinned
    };

    let pawns_double = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::North) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::North) & r_pins & !occupied_all;
        let full_forward = forward_unpinned | forward_pinned;
        (full_forward & bitboard::masks::RANK_3).shift_dir(Direction::North) & !occupied_all
    };

    let p_left_cap_no_prom = pawns_left_captures & !RANK_8;
    let p_left_cap_prom = pawns_left_captures & RANK_8;
    let p_right_cap_no_prom = pawns_right_captures & !RANK_8;
    let p_right_cap_prom = pawns_right_captures & RANK_8;
    let p_forward_no_prom = pawns_forward & !RANK_8;
    let p_forward_prom = pawns_forward & RANK_8;

    let delta = Direction::NorthWest.offset();
    p_left_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_left_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    let delta = Direction::NorthEast.offset();
    p_right_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_right_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    let delta = Direction::North.offset();
    p_forward_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, false));
        on_move(Move::promotion(from, to, Role::Rook, false));
        on_move(Move::promotion(from, to, Role::Bishop, false));
        on_move(Move::promotion(from, to, Role::Knight, false));
    });

    p_forward_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::quiet(from, to));
    });

    let delta = Direction::North.offset();
    pawns_double.for_each(|to| {
        let from = to.offset_checked(-(delta * 2));
        on_move(Move::double_push(from, to));
    });

    if CAPTURES_ONLY {
        (knights & unpinned_m).for_each(|from| {
            let targets = lookup::knight_attacks(from).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (bishops & pinned_m).for_each(|from| {
            let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
            let targets =
                lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e & pin_ray;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (bishops & unpinned_m).for_each(|from| {
            let targets = lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (rooks & pinned_m).for_each(|from| {
            let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
            let targets =
                lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e & pin_ray;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (rooks & unpinned_m).for_each(|from| {
            let targets = lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (queens & pinned_m).for_each(|from| {
            let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
            let targets =
                lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e & pin_ray;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (queens & unpinned_m).for_each(|from| {
            let targets = lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        return;
    }

    (knights & unpinned_m).for_each(|from| {
        let targets = lookup::knight_attacks(from).to_bb() & !occupied_m;
        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (bishops & pinned_m).for_each(|from| {
        let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
        let targets =
            lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m & pin_ray;

        // TODO maybe split into 2 loops for captures and quiet, look into old impl
        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (bishops & unpinned_m).for_each(|from| {
        let targets = lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (rooks & pinned_m).for_each(|from| {
        let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
        let targets =
            lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m & pin_ray;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (rooks & unpinned_m).for_each(|from| {
        let targets = lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (queens & pinned_m).for_each(|from| {
        let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
        let targets =
            lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m & pin_ray;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (queens & unpinned_m).for_each(|from| {
        let targets = lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });
}

fn gen_white_evasion_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    king_sqr: Square,
    ep_sqr: Option<Square>,
    checker: Square,
    pin_info: &PinInfo,
    on_move: &mut impl FnMut(Move),
) {
    let pinned = pin_info.pinned();
    let b_pins = pin_info.b_pins();
    let r_pins = pin_info.r_pins();

    let pawns = board.w_pawns() & !pinned;
    let knights = board.w_knights() & !pinned;
    let bishops = board.w_bishops() & !pinned;
    let rooks = board.w_rooks() & !pinned;
    let queens = board.w_queens() & !pinned;

    let occupied_e = board.blacks();
    let occupied_all = occupied_e | board.whites();

    let evasion_mask = lookup::ray_between(king_sqr, checker).to_bb();

    let capture_mask = checker.to_bb();
    let checker_role = unsafe { board.peek_role_unchecked(checker) };

    let block_or_capture = evasion_mask | capture_mask;

    let pawns_attk = pawns & !r_pins; // rook-pinned pawns can not capture
    let pawns_push = pawns & !b_pins; // diagonally pinned pawns can not push forward

    let pawns_left_captures = {
        let left_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::NorthWest);
        let left_pinned = (pawns_attk & b_pins).shift_dir(Direction::NorthWest) & b_pins;
        let pawns_left_all = left_pinned | left_unpinned;
        pawns_left_all & occupied_e & capture_mask
    };

    let pawns_right_captures = {
        let right_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::NorthEast);
        let right_pinned = (pawns_attk & b_pins).shift_dir(Direction::NorthEast) & b_pins;
        let pawns_right_all = right_pinned | right_unpinned;
        pawns_right_all & occupied_e & capture_mask
    };

    let pawns_forward = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::North) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::North) & r_pins & !occupied_all;
        (forward_unpinned | forward_pinned) & evasion_mask
    };

    let pawns_double = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::North) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::North) & r_pins & !occupied_all;
        let full_forward = forward_unpinned | forward_pinned;
        (full_forward & bitboard::masks::RANK_3).shift_dir(Direction::North)
            & !occupied_all
            & evasion_mask
    };

    let p_left_cap_no_prom = pawns_left_captures & !RANK_8;
    let p_left_cap_prom = pawns_left_captures & RANK_8;
    let p_right_cap_no_prom = pawns_right_captures & !RANK_8;
    let p_right_cap_prom = pawns_right_captures & RANK_8;
    let p_forward_no_prom = pawns_forward & !RANK_8;
    let p_forward_prom = pawns_forward & RANK_8;

    let delta = Direction::NorthWest.offset();
    p_left_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_left_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    let delta = Direction::NorthEast.offset();
    p_right_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_right_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    if checker_role == Role::Pawn {
        if let Some(ep_square) = ep_sqr {
            // find our pawns that could capture onto ep_square (adjacent
            // files, one rank back), excluding ordinary diagonal pins —
            // b_pins already covers "would leave king in check via diagonal"
            let candidates = board.w_pawns() & !r_pins; // rook-pinned pawns can't capture at all

            let ep_bb = Bitboard::from_square(ep_square);

            let left_from = ep_bb.shift_dir(Direction::SouthWest).first_square();
            let right_from = ep_bb.shift_dir(Direction::SouthEast).first_square();

            for maybe_from in [left_from, right_from] {
                let Some(maybe_from) = maybe_from else {
                    continue;
                };

                if !candidates.is_square_set(maybe_from) {
                    continue;
                }
                // ordinary diagonal-pin check: if pinned, capture must stay on the pin ray
                if b_pins.is_square_set(maybe_from) && !b_pins.is_square_set(ep_square) {
                    continue;
                }
                // the special double-pawn-removal pin check
                if white_passant_pin_mask(board, king_sqr, maybe_from, ep_square) == true {
                    continue;
                }

                on_move(Move::en_passant(maybe_from, ep_square));
            }
        }
    }

    if matches!(checker_role, Role::Pawn | Role::Knight) {
        knights.for_each(|from| {
            let targets = lookup::knight_attacks(from).to_bb() & capture_mask;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        bishops.for_each(|from| {
            let targets =
                lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & capture_mask;

            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        rooks.for_each(|from| {
            let targets = lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & capture_mask;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        queens.for_each(|from| {
            let targets = lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & capture_mask;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });
    } else {
        if CAPTURES_ONLY {
            knights.for_each(|from| {
                let targets = lookup::knight_attacks(from).to_bb() & block_or_capture;
                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            bishops.for_each(|from| {
                let targets =
                    lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            rooks.for_each(|from| {
                let targets =
                    lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            queens.for_each(|from| {
                let targets =
                    lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            return;
        }

        let delta = Direction::North.offset();
        p_forward_prom.for_each(|to| {
            let from = to.offset_checked(-delta);
            on_move(Move::promotion(from, to, Role::Queen, false));
            on_move(Move::promotion(from, to, Role::Rook, false));
            on_move(Move::promotion(from, to, Role::Bishop, false));
            on_move(Move::promotion(from, to, Role::Knight, false));
        });

        p_forward_no_prom.for_each(|to| {
            let from = to.offset_checked(-delta);
            on_move(Move::quiet(from, to));
        });

        let delta = Direction::North.offset();
        pawns_double.for_each(|to| {
            let from = to.offset_checked(-(delta * 2));
            on_move(Move::double_push(from, to));
        });

        knights.for_each(|from| {
            let targets = lookup::knight_attacks(from).to_bb() & block_or_capture;
            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });

        bishops.for_each(|from| {
            let targets =
                lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });

        rooks.for_each(|from| {
            let targets =
                lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });

        queens.for_each(|from| {
            let targets =
                lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });
    }
}

fn white_passant_pin_mask(
    board: &Board,
    king_sqr: Square,
    from: Square,
    ep_square: Square,
) -> bool {
    // king must be on the EP capture rank for this to even be possible
    if !bitboard::masks::RANK_5.is_square_set(king_sqr) {
        return false;
    }

    let mut occ_all = board.blacks() | board.whites();

    // the captured black pawn sits one rank behind the ep square
    let captured_pawn = ep_square.offset_checked(-8);

    occ_all = occ_all.clear_square(captured_pawn);
    occ_all = occ_all.clear_square(from);

    let king_rank_attacks = lookup::rook_attacks(king_sqr, occ_all.as_u64()).to_bb();
    let rank_attackers = king_rank_attacks & (board.b_rooks() | board.b_queens());

    if rank_attackers == Bitboard::new_empty() {
        false
    } else {
        true
    }
}

fn gen_white_king_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    king_sqr: Square,
    on_move: &mut impl FnMut(Move),
) {
    let friendly = board.whites();
    let enemy = board.blacks();

    let attk = lookup::king_attacks(king_sqr).to_bb();
    let quiet = attk & !friendly & !enemy;

    if CAPTURES_ONLY {
        let captures = attk & enemy;
        captures.for_each(|to| {
            if white_king_move_is_safe(board, king_sqr, to) {
                on_move(Move::capture(king_sqr, to));
            }
        });
        return;
    }

    let captures = attk & enemy;
    captures.for_each(|to| {
        if white_king_move_is_safe(board, king_sqr, to) {
            on_move(Move::capture(king_sqr, to));
        }
    });

    quiet.for_each(|to| {
        if white_king_move_is_safe(board, king_sqr, to) {
            on_move(Move::quiet(king_sqr, to));
        }
    });
}

unsafe fn gen_legal_moves_black<const CAPTURES_ONLY: bool>(
    pos: &Position,
    on_move: &mut impl FnMut(Move),
) {
    let board = pos.board();
    let king_checkers = pos.checkers_to(Color::Black);
    let king_sqr = board.the_king(Color::Black);
    let ep_sqr = pos.ep_square();

    let pin_info = PinInfo::compute(pos, Color::Black);

    gen_black_king_moves::<CAPTURES_ONLY>(board, king_sqr, on_move);

    if king_checkers.popcnt() == 2 {
        return;
    }

    if king_checkers.popcnt() != 0 {
        let checker = unsafe { king_checkers.first_square_unchecked() };
        gen_black_evasion_moves::<CAPTURES_ONLY>(
            board, king_sqr, ep_sqr, checker, &pin_info, on_move,
        );
        return;
    }

    gen_black_moves::<CAPTURES_ONLY>(board, &pin_info, on_move);
    gen_black_castling_moves::<CAPTURES_ONLY>(board, king_sqr, *pos.castling_rights(), on_move);
    gen_black_ep_moves(pos, king_sqr, on_move);
}

fn gen_black_king_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    king_sqr: Square,
    on_move: &mut impl FnMut(Move),
) {
    let friendly = board.blacks();
    let enemy = board.whites();

    let attacks = lookup::king_attacks(king_sqr).to_bb();

    let quiet = attacks & !friendly & !enemy;

    if CAPTURES_ONLY {
        let captures = attacks & enemy;
        captures.for_each(|to| {
            if black_king_move_is_safe(board, king_sqr, to) {
                on_move(Move::capture(king_sqr, to));
            }
        });
        return;
    }

    let captures = attacks & enemy;
    captures.for_each(|to| {
        if black_king_move_is_safe(board, king_sqr, to) {
            on_move(Move::capture(king_sqr, to));
        }
    });

    quiet.for_each(|to| {
        if black_king_move_is_safe(board, king_sqr, to) {
            on_move(Move::quiet(king_sqr, to));
        }
    });
}

fn gen_black_ep_moves(pos: &Position, king_sqr: Square, on_move: &mut impl FnMut(Move)) {
    let Some(ep) = pos.ep_square() else {
        return;
    };

    let board = pos.board();
    let occupied = board.occupied();

    let pawns = board.b_pawns();

    let enemy_bishops = board.w_bishops();
    let enemy_rooks = board.w_rooks();
    let enemy_queens = board.w_queens();

    let enemies = enemy_bishops | enemy_rooks | enemy_queens;

    let enemy_pawn = Piece::WPawn;

    pawns.for_each(|from| {
        if lookup::pawn_attacks(Color::Black, from) & (1 << ep.as_u32()) == 0 {
            return;
        }

        let captured = Square::of(ep.file(), from.rank());

        if board.peek(captured) != Some(enemy_pawn) {
            return;
        }

        let legal = if enemies.empty() {
            true
        } else {
            let occupied = occupied
                .clear_square(from)
                .clear_square(captured)
                .set_square(ep);

            let opened_enemy_rook =
                lookup::rook_attacks(king_sqr, occupied.as_u64()).to_bb() & enemy_rooks;

            let opened_enemy_bishop =
                lookup::bishop_attacks(king_sqr, occupied.as_u64()).to_bb() & enemy_bishops;

            let opened_enemy_queen =
                lookup::queen_attacks(king_sqr, occupied.as_u64()).to_bb() & enemy_queens;

            (opened_enemy_rook | opened_enemy_bishop | opened_enemy_queen).empty()
        };

        if legal {
            on_move(Move::en_passant(from, ep));
        }
    });
}

fn gen_black_castling_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    king_sqr: Square,
    castling_rights: Castlings,
    on_move: &mut impl FnMut(Move),
) {
    if CAPTURES_ONLY {
        return;
    }

    let occupied = board.occupied();

    if castling_rights.b_short() {
        let f = Square::F8;
        let g = Square::G8;

        let path = f.to_bb().set_square(g);

        if !occupied.intersects(path)
            && !board.is_square_attacked_by(f, Color::White)
            && !board.is_square_attacked_by(g, Color::White)
        {
            on_move(Move::king_castle(king_sqr, g));
        }
    }

    if castling_rights.b_long() {
        let d = Square::D8;
        let c = Square::C8;
        let b = Square::B8;

        let clear = d.to_bb().set_square(c).set_square(b);

        if !occupied.intersects(clear)
            && !board.is_square_attacked_by(d, Color::White)
            && !board.is_square_attacked_by(c, Color::White)
        {
            on_move(Move::queen_castle(king_sqr, c));
        }
    }
}

fn gen_black_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    pin_info: &PinInfo,
    on_move: &mut impl FnMut(Move),
) {
    let pinned = pin_info.pinned();
    let b_pins = pin_info.b_pins();
    let r_pins = pin_info.r_pins();

    let pawns = board.b_pawns();
    let knights = board.b_knights();
    let bishops = board.b_bishops();
    let rooks = board.b_rooks();
    let queens = board.b_queens();

    let occupied_e = board.whites();
    let occupied_m = board.blacks();
    let occupied_all = board.occupied();

    let pinned_m = occupied_m & pinned;
    let unpinned_m = occupied_m & !pinned;

    let pawns_attk = pawns & !r_pins; // rook-pinned pawns can not capture
    let pawns_push = pawns & !b_pins; // diagonally pinned pawns can not push forward

    let pawns_left_captures = {
        let left_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::SouthWest);
        let left_pinned = (pawns_attk & b_pins).shift_dir(Direction::SouthWest) & b_pins;
        let pawns_left_all = left_pinned | left_unpinned;
        pawns_left_all & occupied_e
    };

    let pawns_right_captures = {
        let right_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::SouthEast);
        let right_pinned = (pawns_attk & b_pins).shift_dir(Direction::SouthEast) & b_pins;
        let pawns_right_all = right_pinned | right_unpinned;
        pawns_right_all & occupied_e
    };

    let pawns_forward = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::South) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::South) & r_pins & !occupied_all;
        forward_unpinned | forward_pinned
    };

    let pawns_double = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::South) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::South) & r_pins & !occupied_all;
        let full_forward = forward_unpinned | forward_pinned;
        (full_forward & bitboard::masks::RANK_6).shift_dir(Direction::South) & !occupied_all
    };

    let p_left_cap_no_prom = pawns_left_captures & !RANK_1;
    let p_left_cap_prom = pawns_left_captures & RANK_1;
    let p_right_cap_no_prom = pawns_right_captures & !RANK_1;
    let p_right_cap_prom = pawns_right_captures & RANK_1;
    let p_forward_no_prom = pawns_forward & !RANK_1;
    let p_forward_prom = pawns_forward & RANK_1;

    let delta = Direction::SouthWest.offset();
    p_left_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_left_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    let delta = Direction::SouthEast.offset();
    p_right_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_right_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    let delta = Direction::South.offset();
    p_forward_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, false));
        on_move(Move::promotion(from, to, Role::Rook, false));
        on_move(Move::promotion(from, to, Role::Bishop, false));
        on_move(Move::promotion(from, to, Role::Knight, false));
    });

    p_forward_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::quiet(from, to));
    });

    let delta = Direction::North.offset();
    pawns_double.for_each(|to| {
        let from = to.offset_checked(delta * 2);
        on_move(Move::double_push(from, to));
    });

    if CAPTURES_ONLY {
        (knights & unpinned_m).for_each(|from| {
            let targets = lookup::knight_attacks(from).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (bishops & pinned_m).for_each(|from| {
            let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
            let targets =
                lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e & pin_ray;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (bishops & unpinned_m).for_each(|from| {
            let targets = lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (rooks & pinned_m).for_each(|from| {
            let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
            let targets =
                lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e & pin_ray;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (rooks & unpinned_m).for_each(|from| {
            let targets = lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (queens & pinned_m).for_each(|from| {
            let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
            let targets =
                lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e & pin_ray;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        (queens & unpinned_m).for_each(|from| {
            let targets = lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & occupied_e;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        return;
    }

    (knights & unpinned_m).for_each(|from| {
        let targets = lookup::knight_attacks(from).to_bb() & !occupied_m;
        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (bishops & pinned_m).for_each(|from| {
        let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
        let targets =
            lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m & pin_ray;

        // TODO maybe split into 2 loops for captures and quiet, look into old impl
        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (bishops & unpinned_m).for_each(|from| {
        let targets = lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (rooks & pinned_m).for_each(|from| {
        let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
        let targets =
            lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m & pin_ray;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (rooks & unpinned_m).for_each(|from| {
        let targets = lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (queens & pinned_m).for_each(|from| {
        let pin_ray = unsafe { pin_info.ray_of_unchecked(from) };
        let targets =
            lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m & pin_ray;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });

    (queens & unpinned_m).for_each(|from| {
        let targets = lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & !occupied_m;

        targets.for_each(|to| {
            if occupied_e.is_square_set(to) {
                on_move(Move::capture(from, to));
            } else {
                on_move(Move::quiet(from, to));
            }
        });
    });
}

fn gen_black_evasion_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    king_sqr: Square,
    ep_sqr: Option<Square>,
    checker: Square,
    pin_info: &PinInfo,
    on_move: &mut impl FnMut(Move),
) {
    let pinned = pin_info.pinned();
    let b_pins = pin_info.b_pins();
    let r_pins = pin_info.r_pins();

    let pawns = board.b_pawns() & !pinned;
    let knights = board.b_knights() & !pinned;
    let bishops = board.b_bishops() & !pinned;
    let rooks = board.b_rooks() & !pinned;
    let queens = board.b_queens() & !pinned;

    let occupied_e = board.whites();
    let occupied_all = occupied_e | board.blacks();

    let evasion_mask = lookup::ray_between(king_sqr, checker).to_bb();

    let capture_mask = checker.to_bb();
    let checker_role = unsafe { board.peek_role_unchecked(checker) };

    let block_or_capture = evasion_mask | capture_mask;

    let pawns_attk = pawns & !r_pins; // rook-pinned pawns can not capture
    let pawns_push = pawns & !b_pins; // diagonally pinned pawns can not push forward

    let pawns_left_captures = {
        let left_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::SouthWest);
        let left_pinned = (pawns_attk & b_pins).shift_dir(Direction::SouthWest) & b_pins;
        let pawns_left_all = left_pinned | left_unpinned;
        pawns_left_all & occupied_e & capture_mask
    };

    let pawns_right_captures = {
        let right_unpinned = (pawns_attk & !b_pins).shift_dir(Direction::SouthEast);
        let right_pinned = (pawns_attk & b_pins).shift_dir(Direction::SouthEast) & b_pins;
        let pawns_right_all = right_pinned | right_unpinned;
        pawns_right_all & occupied_e & capture_mask
    };

    let pawns_forward = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::South) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::South) & r_pins & !occupied_all;
        (forward_unpinned | forward_pinned) & evasion_mask
    };

    let pawns_double = if CAPTURES_ONLY {
        Bitboard::new_empty()
    } else {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::South) & !occupied_all;
        let forward_pinned =
            (pawns_push & r_pins).shift_dir(Direction::South) & r_pins & !occupied_all;
        let full_forward = forward_unpinned | forward_pinned;
        (full_forward & bitboard::masks::RANK_6).shift_dir(Direction::South)
            & !occupied_all
            & evasion_mask
    };

    let p_left_cap_no_prom = pawns_left_captures & !RANK_1;
    let p_left_cap_prom = pawns_left_captures & RANK_1;
    let p_right_cap_no_prom = pawns_right_captures & !RANK_1;
    let p_right_cap_prom = pawns_right_captures & RANK_1;
    let p_forward_no_prom = pawns_forward & !RANK_1;
    let p_forward_prom = pawns_forward & RANK_1;

    let delta = Direction::SouthWest.offset();
    p_left_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_left_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    let delta = Direction::SouthEast.offset();
    p_right_cap_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::promotion(from, to, Role::Queen, true));
        on_move(Move::promotion(from, to, Role::Rook, true));
        on_move(Move::promotion(from, to, Role::Bishop, true));
        on_move(Move::promotion(from, to, Role::Knight, true));
    });

    p_right_cap_no_prom.for_each(|to| {
        let from = to.offset_checked(-delta);
        on_move(Move::capture(from, to));
    });

    if checker_role == Role::Pawn {
        if let Some(ep_square) = ep_sqr {
            // find our pawns that could capture onto ep_square (adjacent
            // files, one rank back), excluding ordinary diagonal pins —
            // b_pins already covers "would leave king in check via diagonal"
            let candidates = board.b_pawns() & !r_pins; // rook-pinned pawns can't capture at all

            let ep_bb = Bitboard::from_square(ep_square);

            let left_from = ep_bb.shift_dir(Direction::NorthWest).first_square();
            let right_from = ep_bb.shift_dir(Direction::NorthEast).first_square();

            for maybe_from in [left_from, right_from] {
                let Some(maybe_from) = maybe_from else {
                    continue;
                };

                if !candidates.is_square_set(maybe_from) {
                    continue;
                }

                // ordinary diagonal-pin check: if pinned, capture must stay on the pin ray
                if b_pins.is_square_set(maybe_from) && !b_pins.is_square_set(ep_square) {
                    continue;
                }
                // the special double-pawn-removal pin check
                if black_passant_pin_mask(board, king_sqr, maybe_from, ep_square) == true {
                    continue;
                }

                on_move(Move::en_passant(maybe_from, ep_square));
            }
        }
    }

    if matches!(checker_role, Role::Pawn | Role::Knight) {
        knights.for_each(|from| {
            let targets = lookup::knight_attacks(from).to_bb() & capture_mask;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        bishops.for_each(|from| {
            let targets =
                lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & capture_mask;

            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        rooks.for_each(|from| {
            let targets = lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & capture_mask;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });

        queens.for_each(|from| {
            let targets = lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & capture_mask;
            targets.for_each(|to| {
                on_move(Move::capture(from, to));
            });
        });
    } else {
        let delta = Direction::South.offset();
        if CAPTURES_ONLY {
            knights.for_each(|from| {
                let targets = lookup::knight_attacks(from).to_bb() & block_or_capture;
                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            bishops.for_each(|from| {
                let targets =
                    lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            rooks.for_each(|from| {
                let targets =
                    lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            queens.for_each(|from| {
                let targets =
                    lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

                targets.for_each(|to| {
                    if occupied_e.is_square_set(to) {
                        on_move(Move::capture(from, to));
                    }
                });
            });

            return;
        }

        p_forward_prom.for_each(|to| {
            let from = to.offset_checked(-delta);
            on_move(Move::promotion(from, to, Role::Queen, false));
            on_move(Move::promotion(from, to, Role::Rook, false));
            on_move(Move::promotion(from, to, Role::Bishop, false));
            on_move(Move::promotion(from, to, Role::Knight, false));
        });

        p_forward_no_prom.for_each(|to| {
            let from = to.offset_checked(-delta);
            on_move(Move::quiet(from, to));
        });

        let delta = Direction::South.offset();
        pawns_double.for_each(|to| {
            let from = to.offset_checked(-(delta * 2));
            on_move(Move::double_push(from, to));
        });

        knights.for_each(|from| {
            let targets = lookup::knight_attacks(from).to_bb() & block_or_capture;
            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });

        bishops.for_each(|from| {
            let targets =
                lookup::bishop_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });

        rooks.for_each(|from| {
            let targets =
                lookup::rook_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });

        queens.for_each(|from| {
            let targets =
                lookup::queen_attacks(from, occupied_all.as_u64()).to_bb() & block_or_capture;

            targets.for_each(|to| {
                if occupied_e.is_square_set(to) {
                    on_move(Move::capture(from, to));
                } else {
                    on_move(Move::quiet(from, to));
                }
            });
        });
    }
}

fn black_passant_pin_mask(
    board: &Board,
    king_sqr: Square,
    from: Square,
    ep_square: Square,
) -> bool {
    // king must be on the EP capture rank for this to even be possible
    if !bitboard::masks::RANK_4.is_square_set(king_sqr) {
        return false;
    }

    let mut occ_all = board.blacks() | board.whites();

    // the captured black pawn sits one rank behind the ep square
    let captured_pawn = ep_square.offset_checked(Direction::North.offset());

    occ_all = occ_all.clear_square(captured_pawn);
    occ_all = occ_all.clear_square(from);

    let king_rank_attacks = lookup::rook_attacks(king_sqr, occ_all.as_u64()).to_bb();
    let rank_attackers = king_rank_attacks & (board.w_rooks() | board.w_queens());

    if rank_attackers == Bitboard::new_empty() {
        false
    } else {
        true
    }
}
fn white_king_move_is_safe(board: &Board, from: Square, to: Square) -> bool {
    let us = Color::White;

    let occupied = board.occupied().clear_square(from).set_square(to);

    let queens = board.b_queens();
    let pawns = board.b_pawns();
    let knights = board.b_knights();
    let bishops = board.b_bishops() | queens;
    let rooks = board.b_rooks() | queens;
    let king = board.b_king();

    (lookup::pawn_attacks(us, to).to_bb() & pawns).empty()
        && (lookup::knight_attacks(to).to_bb() & knights).empty()
        && (lookup::bishop_attacks(to, occupied.as_u64()).to_bb() & bishops).empty()
        && (lookup::rook_attacks(to, occupied.as_u64()).to_bb() & rooks).empty()
        && (lookup::king_attacks(to).to_bb() & king).empty()
}

fn black_king_move_is_safe(board: &Board, from: Square, to: Square) -> bool {
    let us = Color::Black;

    let occupied = board.occupied().clear_square(from).set_square(to);

    let queens = board.w_queens();
    let pawns = board.w_pawns();
    let knights = board.w_knights();
    let bishops = board.w_bishops() | queens;
    let rooks = board.w_rooks() | queens;
    let king = board.w_king();

    (lookup::pawn_attacks(us, to).to_bb() & pawns).empty()
        && (lookup::knight_attacks(to).to_bb() & knights).empty()
        && (lookup::bishop_attacks(to, occupied.as_u64()).to_bb() & bishops).empty()
        && (lookup::rook_attacks(to, occupied.as_u64()).to_bb() & rooks).empty()
        && (lookup::king_attacks(to).to_bb() & king).empty()
}

mod pin_info {
    use std::{hint::unreachable_unchecked, mem::MaybeUninit};

    use types::{
        bitboard::{Bitboard, ToBitboard},
        color::Color,
        square::Square,
    };

    use crate::position::Position;

    #[derive(Debug)]
    pub struct PinInfo {
        b_pins: Bitboard,
        r_pins: Bitboard,
        pin_rays: [MaybeUninit<Bitboard>; 8],
        squares: [MaybeUninit<Square>; 8],
        len: u8,
    }

    impl PinInfo {
        // SAFETY: It's caller's responsibility to ensure that the entry for requested `Square` has been set
        pub unsafe fn ray_of_unchecked(&self, square: Square) -> Bitboard {
            for i in 0..self.len as usize {
                if unsafe { self.squares.get_unchecked(i).assume_init() } == square {
                    return unsafe { self.pin_rays.get_unchecked(i).assume_init() };
                }
            }

            unsafe { unreachable_unchecked() };
        }

        pub fn ray_of(&self, square: Square) -> Option<Bitboard> {
            if self.pinned().is_square_set(square) {
                // SAFETY: calling this unsafe function is sound because square is pinned
                Some(unsafe { self.ray_of_unchecked(square) })
            } else {
                None
            }
        }

        pub fn pinned(&self) -> Bitboard {
            self.b_pins | self.r_pins
        }

        pub fn b_pins(&self) -> Bitboard {
            self.b_pins
        }

        pub fn r_pins(&self) -> Bitboard {
            self.r_pins
        }

        pub fn compute(pos: &Position, us: Color) -> Self {
            let them = !us;

            let board = pos.board();
            let occupied = board.occupied();
            let king = board.the_king(us);

            let enemy_bishops = board.bishops(them) | board.queens(them);
            let enemy_rooks = board.rooks(them) | board.queens(them);

            let mut b_pins = 0_u64.to_bb();
            let mut r_pins = 0_u64.to_bb();
            let mut pin_rays = [MaybeUninit::<Bitboard>::uninit(); 8];
            let mut squares = [MaybeUninit::<Square>::uninit(); 8];
            let mut len = 0;

            let diagonal_attacks = lookup::diagonal_rays_from(king).to_bb() & enemy_bishops;
            diagonal_attacks.for_each(|attacker| {
                do_compute(
                    occupied,
                    king,
                    &mut b_pins,
                    &mut pin_rays,
                    &mut squares,
                    &mut len,
                    attacker,
                );
            });

            let orthogonal_attacks = lookup::orthogonal_rays_from(king).to_bb() & enemy_rooks;
            orthogonal_attacks.for_each(|attacker| {
                do_compute(
                    occupied,
                    king,
                    &mut r_pins,
                    &mut pin_rays,
                    &mut squares,
                    &mut len,
                    attacker,
                );
            });

            Self {
                b_pins,
                r_pins,
                pin_rays,
                squares,
                len,
            }
        }
    }

    fn do_compute(
        occupied: Bitboard,
        king: Square,
        pinned_pieces: &mut Bitboard,
        pin_rays: &mut [MaybeUninit<Bitboard>; 8],
        squares: &mut [MaybeUninit<Square>; 8],
        len: &mut u8,
        attacker: Square,
    ) {
        let between = lookup::ray_between(king, attacker).to_bb();
        let between_inclusive = between | king.to_bb() | attacker.to_bb();
        let blockers = between & occupied;

        if let Some(blocker_sqr) = blockers.only_first_square() {
            *pinned_pieces = *pinned_pieces | between_inclusive;
            // SAFETY: Max number of pinned pieces is 8, hence len is always between 0 and 8
            unsafe {
                pin_rays
                    .get_unchecked_mut(*len as usize)
                    .write(between_inclusive);

                squares.get_unchecked_mut(*len as usize).write(blocker_sqr);
            };

            *len += 1;
        }
    }
}
