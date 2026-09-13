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
    rank::Rank,
    role::Role,
    square::Square,
};

use crate::{board::Board, move_gen::pin_info::PinInfo, position::Position};

fn gen_legal_moves_for_v2(pos: &Position, color: Color) -> MoveList {
    let mut moves = MoveList::new();
    let on_move = |mv| unsafe { moves.push_unchecked(mv) };

    unsafe {
        match color {
            Color::White => gen_legal_moves_white(pos, on_move),
            Color::Black => gen_legal_moves_black(pos, on_move),
        }
    }
    moves
}

unsafe fn gen_legal_moves_white(pos: &Position, on_move: impl FnMut(Move) -> ()) {
    let board = pos.board();
    let king_checkers = pos.checkers_to(Color::White);
    let king_sqr = board.the_king(Color::White);

    let pin_info = PinInfo::compute(pos, Color::White);

    if king_checkers.popcnt() == 2 {
        // gen_king_moves_white::<false>(board, king_sqr, on_move);
        return;
    }

    if king_checkers.popcnt() != 0 {
        let checker = unsafe { king_checkers.first_square_unchecked() };
        gen_white_evasion_moves::<false>(board, king_sqr, checker, &pin_info, on_move);
        return;
    }

    // gen_white_moves::<false>(board, &pin_info, on_move);
    // gen_castling_moves_white::<false>(board, *pos.castling_rights(), on_move);
    // gen_ep_moves_white::<false>(pos, on_move);
}

fn gen_white_evasion_moves<const CAPTURES_ONLY: bool>(
    board: &Board,
    king_sqr: Square,
    checker: Square,
    pin_info: &PinInfo,
    on_move: impl FnMut(Move),
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

    let evasion_mask = lookup::ray_between(king_sqr, checker);

    let capture_mask = checker.to_bb();
    let checker_role = unsafe { board.peek_role_unchecked(checker) };

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

    let pawns_forward = {
        let forward_unpinned = (pawns_push & !r_pins).shift_dir(Direction::North) & !occupied_all;
        let forward_pinned = (pawns_push & r_pins).shift_dir(Direction::North) & r_pins;
        forward_unpinned | forward_pinned
    };

    let pawns_double =
        (pawns_forward & bitboard::masks::RANK_3).shift_dir(Direction::North) & !occupied_all;

    // for pawn in pawns_left_captures {
    //     on_move(Move::capture(Role::Pawn, from, pawn, capture))
    // }
}

unsafe fn gen_legal_moves_black(pos: &Position, on_move: impl FnMut(Move) -> ()) {
    todo!()
}

pub fn gen_legal_moves_for(pos: &Position, us: Color) -> MoveList {
    let mut moves = MoveList::new();

    let mut on_move = |mv| unsafe { moves.push_unchecked(mv) };
    
    let board = pos.board();
    let king_checkers = pos.checkers_to(us);
    let king_sqr = board.the_king(us);

    let pin_info = PinInfo::compute(pos, us);

    if king_checkers.empty() {
        gen_quiet_and_captures(board, us, &pin_info, &mut on_move);
        gen_castling_moves(board, *pos.castling_rights(), us, &mut on_move);
        gen_ep_moves(pos, us, &mut on_move);
    } else if king_checkers.popcnt() == 2 {
        gen_king_moves(board, us, king_sqr, &mut on_move);
    } else {
        gen_evasions(
            pos,
            us,
            king_sqr,
            king_checkers,
            pin_info.pinned(),
            &mut on_move,
        );
    }

    moves
}

fn gen_quiet_and_captures(board: &Board, us: Color, pin_info: &PinInfo, on_move: &mut impl FnMut(Move)) {
    gen_king_moves(board, us, board.the_king(us),  on_move);

    let pawns = board.pawns(us);
    let knights = board.knights(us);
    let bishops = board.bishops(us);
    let rooks = board.rooks(us);
    let queens = board.queens(us);

    let pinned = pin_info.pinned();
    let unpinned = board.by_color(us) & !pinned;

    gen_unpinned_quiet_and_capture_moves(
        board,
        us,
        pawns & unpinned,
        knights & unpinned,
        bishops & unpinned,
        rooks & unpinned,
        queens & unpinned,
        on_move,
    );

    gen_pinned_quiet_and_capture_moves(
        board,
        us,
        pawns & pinned,
        bishops & pinned,
        rooks & pinned,
        queens & pinned,
        pin_info,
        on_move,
    );
}

#[allow(clippy::too_many_arguments)]
fn gen_pinned_quiet_and_capture_moves(
    board: &Board,
    us: Color,
    pawns: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    pin_info: &PinInfo,
    on_move: &mut impl FnMut(Move),
) {
    let occupied = board.occupied();
    let enemy = board.by_color(!us);
    let empty = !occupied;

    pawns.for_each(|pawn| {
        let Some(pin_ray) = pin_info.ray_of(pawn) else {
            return;
        };

        let restricted_pawn_attacks = pin_ray & lookup::pawn_attacks(us, pawn).to_bb();
        restricted_pawn_attacks.for_each(|attk| {
            let maybe_enemy = board.peek(attk);
            if let Some(enemy_piece) = maybe_enemy
                && enemy_piece.color() == !us
            {
                // capture promotion
                if attk.rank() == Rank::First || attk.rank() == Rank::Eighth {
                    on_move(Move::promotion(pawn, attk, Role::Queen, true));
                    on_move(Move::promotion(pawn, attk, Role::Rook, true));
                    on_move(Move::promotion(pawn, attk, Role::Bishop, true));
                    on_move(Move::promotion(pawn, attk, Role::Knight, true));
                } else {
                    on_move(Move::capture(pawn, attk));
                }
            }
        });

        let restricted_pawn_pushes = pin_ray & lookup::pawn_pushes(us, pawn).to_bb();
        restricted_pawn_pushes.for_each(|push| {
            let not_occupied = !occupied.is_square_set(push);
            if not_occupied {
                if push.rank() == Rank::First || push.rank() == Rank::Eighth {
                    on_move(Move::promotion(pawn, push, Role::Queen, false));
                    on_move(Move::promotion(pawn, push, Role::Rook, false));
                    on_move(Move::promotion(pawn, push, Role::Bishop, false));
                    on_move(Move::promotion(pawn, push, Role::Knight, false));
                } else {
                    on_move(Move::quiet(pawn, push));
                }
            }
        });

        let restricted_double_pushes = pin_ray & lookup::pawn_double_pushes(us, pawn).to_bb();
        restricted_double_pushes.for_each(|double_push| {
            let not_occupied = !occupied.is_square_set(double_push);
            let mid_not_occupied = !occupied.is_square_set(if us == Color::White {
                double_push.offset_checked(-8)
            } else {
                double_push.offset_checked(8)
            });

            if not_occupied && mid_not_occupied {
                on_move(Move::double_push(pawn, double_push));
            }
        });
    });

    bishops.for_each(|from| {
        let Some(pin_ray) = pin_info.ray_of(from) else {
            return;
        };

        let attacks = lookup::bishop_attacks(from, occupied.as_u64()).to_bb() & pin_ray;

        let capture = attacks & enemy;
        let quiet = attacks & empty;

        quiet.for_each(|to| {
            on_move(Move::quiet(from, to));
        });

        capture.for_each(|to| {
            on_move(Move::capture(from, to));
        });
    });

    rooks.for_each(|from| {
        let Some(pin_ray) = pin_info.ray_of(from) else {
            return;
        };

        let attacks = lookup::rook_attacks(from, occupied.as_u64()).to_bb() & pin_ray;

        let capture = attacks & enemy;
        let quiet = attacks & empty;

        quiet.for_each(|to| {
            on_move(Move::quiet(from, to));
        });

        capture.for_each(|to| {
            on_move(Move::capture(from, to));
        });
    });

    queens.for_each(|from| {
        let Some(pin_ray) = pin_info.ray_of(from) else {
            return;
        };

        let attacks = lookup::queen_attacks(from, occupied.as_u64()).to_bb() & pin_ray;

        let capture = attacks & enemy;
        let quiet = attacks & empty;

        quiet.for_each(|to| {
            on_move(Move::quiet(from, to));
        });

        capture.for_each(|to| {
            on_move(Move::capture(from, to));
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn gen_unpinned_quiet_and_capture_moves(
    board: &Board,
    us: Color,
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    on_move: &mut impl FnMut(Move),
) {
    let friendly = board.by_color(us);
    let enemy = board.by_color(!us);
    let occupied = board.occupied();
    let empty = !occupied;

    let (
        push_dir,
        push_dir_invert_offset,
        cap_left_dir,
        cap_left_dir_invert_offset,
        cap_right_dir,
        cap_right_dir_invert_offset,
        double_push_rank,
        prom_rank,
    ) = match us {
        Color::White => (
            Direction::North,
            Direction::North.invert().offset(),
            Direction::NorthWest,
            Direction::NorthWest.invert().offset(),
            Direction::NorthEast,
            Direction::NorthEast.invert().offset(),
            RANK_2,
            RANK_8,
        ),
        Color::Black => (
            Direction::South,
            Direction::South.invert().offset(),
            Direction::SouthWest,
            Direction::SouthWest.invert().offset(),
            Direction::SouthEast,
            Direction::SouthEast.invert().offset(),
            RANK_7,
            RANK_1,
        ),
    };

    let push_dir_invert_offset_2x = push_dir_invert_offset * 2;

    let single = pawns.shift_dir(push_dir) & empty;
    let quiet = single & !prom_rank;

    quiet.for_each(|from| {
        on_move(Move::quiet(
            from.offset_checked(push_dir_invert_offset),
            from,
        ));
    });

    let double =
        ((pawns & double_push_rank).shift_dir(push_dir) & empty).shift_dir(push_dir) & empty;

    double.for_each(|from| {
        on_move(Move::double_push(
            from.offset_checked(push_dir_invert_offset_2x),
            from,
        ));
    });

    let cap_left = pawns.shift_dir(cap_left_dir) & enemy;
    let cap_right = pawns.shift_dir(cap_right_dir) & enemy;

    let no_prom_cap_left = cap_left & !prom_rank;
    no_prom_cap_left.for_each(|cap| {
        on_move(Move::capture(
            cap.offset_checked(cap_left_dir_invert_offset),
            cap,
        ));
    });

    let no_prom_cap_right = cap_right & !prom_rank;
    no_prom_cap_right.for_each(|cap| {
        on_move(Move::capture(
            cap.offset_checked(cap_right_dir_invert_offset),
            cap,
        ));
    });

    let prom_push = single & prom_rank;
    prom_push.for_each(|promo| {
        let prom_sqr_orig = promo.offset_checked(push_dir_invert_offset);
        on_move(Move::promotion(prom_sqr_orig, promo, Role::Queen, false));
        on_move(Move::promotion(prom_sqr_orig, promo, Role::Rook, false));
        on_move(Move::promotion(prom_sqr_orig, promo, Role::Bishop, false));
        on_move(Move::promotion(prom_sqr_orig, promo, Role::Knight, false));
    });

    let cap_left_prom = cap_left & prom_rank;
    cap_left_prom.for_each(|cap_prom| {
        let cap_prom_sqr_orig = cap_prom.offset_checked(cap_left_dir_invert_offset);

        let prom_q = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Queen, true);
        let prom_r = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Rook, true);
        let prom_b = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Bishop, true);
        let prom_n = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Knight, true);

        on_move(prom_q);
        on_move(prom_r);
        on_move(prom_b);
        on_move(prom_n);
    });

    let cap_right_prom = cap_right & prom_rank;
    cap_right_prom.for_each(|cap_prom| {
        let cap_prom_sqr_orig = cap_prom.offset_checked(cap_right_dir_invert_offset);

        let prom_q = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Queen, true);
        let prom_r = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Rook, true);
        let prom_b = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Bishop, true);
        let prom_n = Move::promotion(cap_prom_sqr_orig, cap_prom, Role::Knight, true);

        on_move(prom_q);
        on_move(prom_r);
        on_move(prom_b);
        on_move(prom_n);
    });

    knights.for_each(|from| {
        let attacks = lookup::knight_attacks(from).to_bb();

        let quiet = attacks & !friendly & !enemy;
        quiet.for_each(|to| {
            on_move(Move::quiet(from, to));
        });

        let captures = attacks & enemy;
        captures.for_each(|to| {
            on_move(Move::capture(from, to));
        });
    });

    bishops.for_each(|from| {
        let attacks = lookup::bishop_attacks(from, occupied.as_u64()).to_bb();
        let targets = attacks & !friendly;

        let captures = targets & enemy;
        let quiet = targets & empty;

        quiet.for_each(|to| {
            on_move(Move::quiet(from, to));
        });

        captures.for_each(|to| {
            on_move(Move::capture(from, to));
        });
    });

    rooks.for_each(|from| {
        let attacks = lookup::rook_attacks(from, occupied.as_u64()).to_bb();
        let targets = attacks & !friendly;

        let captures = targets & enemy;
        let quiet = targets & empty;

        quiet.for_each(|to| {
            on_move(Move::quiet(from, to));
        });

        captures.for_each(|to| {
            on_move(Move::capture(from, to));
        });
    });

    queens.for_each(|from| {
        let attacks = lookup::queen_attacks(from, occupied.as_u64()).to_bb();
        let targets = attacks & !friendly;

        let captures = targets & enemy;
        let quiet = targets & empty;

        quiet.for_each(|to| {
            on_move(Move::quiet(from, to));
        });

        captures.for_each(|to| {
            on_move(Move::capture(from, to));
        });
    });
}

fn gen_castling_moves(board: &Board, castling_rights: Castlings, us: Color, mut on_move: impl FnMut(Move)) {
    let enemy = !us;

    let king_sqr = board.the_king(us);

    let occupied = board.occupied();

    if castling_rights.short(us) {
        let f = Square::from_u32_checked(king_sqr.as_u32() + 1);
        let g = Square::from_u32_checked(king_sqr.as_u32() + 2);

        let path = f.to_bb().set_square(g);

        if !occupied.intersects(path)
            && !board.is_square_attacked_by(f, enemy)
            && !board.is_square_attacked_by(g, enemy)
        {
            on_move(Move::king_castle(king_sqr, g));
        }
    }

    if castling_rights.long(us) {
        let d = Square::from_u32_checked(king_sqr.as_u32() - 1);
        let c = Square::from_u32_checked(king_sqr.as_u32() - 2);
        let b = Square::from_u32_checked(king_sqr.as_u32() - 3);

        let clear = d.to_bb().set_square(c).set_square(b);

        if !occupied.intersects(clear)
            && !board.is_square_attacked_by(d, enemy)
            && !board.is_square_attacked_by(c, enemy)
        {
            on_move(Move::queen_castle(king_sqr, c));
        }
    }
}

pub fn gen_ep_moves(pos: &Position, us: Color, on_move: &mut impl FnMut(Move)) {
    let Some(ep) = pos.ep_square() else {
        return;
    };

    let board = pos.board();
    let occupied = board.occupied();

    let enemy = !us;

    let king_sqr = board.the_king(us);
    let pawns = board.pawns(us);

    let enemy_bishops = board.bishops(enemy);
    let enemy_rooks = board.rooks(enemy);
    let enemy_queens = board.queens(enemy);

    let enemies = enemy_bishops | enemy_rooks | enemy_queens;

    let enemy_pawn = Piece::of(Role::Pawn, enemy);

    pawns.for_each(|from| {
        if lookup::pawn_attacks(us, from) & (1 << ep.as_u32()) == 0 {
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

fn gen_evasions(
    pos: &Position,
    us: Color,
    king_sqr: Square,
    checker: Bitboard,
    pinned: Bitboard,
    on_move: &mut impl FnMut(Move),
) {
    let board = pos.board();

    gen_king_moves(board, us, king_sqr, on_move);

    // although we already know the existence of checker due to call site
    // let-else is more performant than panicking alternative
    let Some(checker) = checker.only_first_square() else {
        return;
    };

    let evasion_mask = lookup::ray_between(king_sqr, checker).to_bb();

    gen_blocking_moves(board, us, evasion_mask, checker, pinned, on_move);
    gen_ep_evasions(pos, us, checker, pinned, on_move);
}

// NOTE consider calculating via enemy piece attack maps in order to avoid branches
fn gen_king_moves(board: &Board, us: Color, king_sqr: Square, on_move: &mut impl FnMut(Move)) {
    let friendly = board.by_color(us);
    let enemy = board.by_color(!us);

    let attacks = lookup::king_attacks(king_sqr).to_bb();

    let quiet = attacks & !friendly & !enemy;
    quiet.for_each(|to| {
        if king_move_is_safe(board, us, king_sqr, to) {
            on_move(Move::quiet(king_sqr, to));
        }
    });

    let captures = attacks & enemy;
    captures.for_each(|to| {
        if king_move_is_safe(board, us, king_sqr, to) {
            on_move(Move::capture(king_sqr, to));
        }
    });
}

fn gen_blocking_moves(
    board: &Board,
    us: Color,
    evasion_mask: Bitboard,
    checker: Square,
    pinned: Bitboard,
    on_move: &mut impl FnMut(Move),
) {
    let occupied = board.occupied();

    let rooks = board.rooks(us) & !pinned;
    let bishops = board.bishops(us) & !pinned;
    let queens = board.queens(us) & !pinned;
    let knights = board.knights(us) & !pinned;
    let pawns = board.pawns(us) & !pinned;

    let capture_mask = checker.to_bb();

    pawns.for_each(|pawn| {
        let pawn_pushes = lookup::pawn_pushes(us, pawn).to_bb() & evasion_mask;
        pawn_pushes.for_each(|push| {
            let promotion_rank = push.rank() == Rank::First || push.rank() == Rank::Eighth;
            if promotion_rank {
                for role in [Role::Queen, Role::Rook, Role::Bishop, Role::Knight] {
                    on_move(Move::promotion(pawn, push, role, false));
                }
            } else {
                on_move(Move::quiet(pawn, push));
            }
        });

        let pawn_double_pushes = lookup::pawn_double_pushes(us, pawn).to_bb() & evasion_mask;
        pawn_double_pushes.for_each(|push| {
            if occupied.is_square_set(push.offset_checked(if us == Color::White { -8 } else { 8 }))
            {
                return;
            }

            on_move(Move::double_push(pawn, push));
        });

        let pawn_attacks = lookup::pawn_attacks(us, pawn).to_bb() & checker.to_bb();
        pawn_attacks.for_each(|_| {
            let promotion_rank = checker.rank() == Rank::First || checker.rank() == Rank::Eighth;

            if promotion_rank {
                for role in [Role::Queen, Role::Rook, Role::Bishop, Role::Knight] {
                    on_move(Move::promotion(pawn, checker, role, true));
                }
            } else {
                on_move(Move::capture(pawn, checker));
            }
        });
    });

    let ray_mask = capture_mask | evasion_mask;
    ray_mask.for_each(|attk| {
        let queen_attackers = lookup::queen_attacks(attk, occupied.as_u64()).to_bb() & queens;
        queen_attackers.for_each(|queen| {
            if attk == checker {
                on_move(Move::capture(queen, attk));
            } else {
                on_move(Move::quiet(queen, attk));
            }
        });

        let rook_atackers = lookup::rook_attacks(attk, occupied.as_u64()).to_bb() & rooks;
        rook_atackers.for_each(|rook| {
            if attk == checker {
                on_move(Move::capture(rook, attk));
            } else {
                on_move(Move::quiet(rook, attk));
            }
        });

        let bishop_attackers = lookup::bishop_attacks(attk, occupied.as_u64()).to_bb() & bishops;
        bishop_attackers.for_each(|bishop| {
            if attk == checker {
                on_move(Move::capture(bishop, attk));
            } else {
                on_move(Move::quiet(bishop, attk));
            }
        });

        let knight_attackers = lookup::knight_attacks(attk).to_bb() & knights;
        knight_attackers.for_each(|knight| {
            if attk == checker {
                on_move(Move::capture(knight, attk));
            } else {
                on_move(Move::quiet(knight, attk));
            }
        });
    });
}

fn gen_ep_evasions(
    pos: &Position,
    us: Color,
    checker: Square,
    pinned: Bitboard,
    on_move: &mut impl FnMut(Move),
) {
    let Some(ep_sqr) = pos.ep_square() else {
        return;
    };

    let board = pos.board();

    let target_pawn = Square::of(
        ep_sqr.file(),
        if us == Color::White {
            Rank::Fifth
        } else {
            Rank::Fourth
        },
    );

    if target_pawn != checker {
        return;
    }

    let pawns = board.pawns(us) & lookup::pawn_attacks(!us, ep_sqr).to_bb();
    let king_sqr = board.the_king(us);

    pawns.for_each(|pawn| {
        if pinned.is_square_set(pawn) {
            return;
        }

        let enemy_sliders = board.rooks(!us) | board.queens(!us);

        let legal = if enemy_sliders.empty() {
            true
        } else {
            let occupied = pos
                .board()
                .occupied()
                .clear_square(pawn)
                .clear_square(target_pawn)
                .set_square(ep_sqr);

            (lookup::rook_attacks(king_sqr, occupied.as_u64()).to_bb() & enemy_sliders).empty()
        };

        if legal {
            on_move(Move::en_passant(pawn, ep_sqr));
        }
    });
}

// FINAL: no need to optimize further
fn king_move_is_safe(board: &Board, us: Color, from: Square, to: Square) -> bool {
    let enemy = !us;

    let occupied = board.occupied().clear_square(from).set_square(to);

    let queens = board.queens(enemy);
    let pawns = board.pawns(enemy);
    let knights = board.knights(enemy);
    let bishops = board.bishops(enemy) | queens;
    let rooks = board.rooks(enemy) | queens;
    let king = board.king(enemy);

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
            *pinned_pieces = pinned_pieces.set_square(blocker_sqr);
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
