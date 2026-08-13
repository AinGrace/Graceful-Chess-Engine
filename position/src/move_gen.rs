use types::{
    MoveList,
    bitboard::{
        Bitboard, ToBitboard,
        masks::{RANK_1, RANK_2, RANK_7, RANK_8},
    },
    chess_move::Move,
    color::Color,
    direction::Direction,
    piece::Piece,
    rank::Rank,
    role::Role,
    square::Square,
};

use crate::{move_gen::pin_info::PinInfo, position::Position};

pub fn gen_legal_moves_for(pos: &Position, us: Color) -> MoveList {
    let mut moves = MoveList::new();
    let king_checkers = pos.checkers_to(us);
    let king_sqr = pos.board().the_king(us);

    let pin_info = PinInfo::compute(pos, us);

    if king_checkers.empty() {
        gen_quiet_and_captures(pos, us, &pin_info, &mut moves);
        gen_castling_moves(pos, us, &mut moves);
        gen_ep_moves(pos, us, &mut moves);
    } else if king_checkers.popcnt() == 2 {
        gen_king_moves(pos, us, king_sqr, &mut moves);
    } else {
        gen_evasions(
            pos,
            us,
            king_sqr,
            king_checkers,
            pin_info.pinned(),
            &mut moves,
        );
    }

    moves
}

fn gen_quiet_and_captures(pos: &Position, us: Color, pin_info: &PinInfo, moves: &mut MoveList) {
    let board = pos.board();

    gen_king_moves(pos, us, board.the_king(us), moves);

    let pawns = board.pawns(us);
    let knights = board.knights(us);
    let bishops = board.bishops(us);
    let rooks = board.rooks(us);
    let queens = board.queens(us);

    let pinned = pin_info.pinned();
    let unpinned = board.by_color(us) & !pinned;

    gen_unpinned_quiet_and_capture_moves(
        pos,
        us,
        pawns & unpinned,
        knights & unpinned,
        bishops & unpinned,
        rooks & unpinned,
        queens & unpinned,
        moves,
    );

    gen_pinned_quiet_and_capture_moves(
        pos,
        us,
        pawns & pinned,
        bishops & pinned,
        rooks & pinned,
        queens & pinned,
        pin_info,
        moves,
    );
}

#[allow(clippy::too_many_arguments)]
fn gen_pinned_quiet_and_capture_moves(
    pos: &Position,
    us: Color,
    pawns: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    pin_info: &PinInfo,
    moves: &mut MoveList,
) {
    let occupied = pos.board().occupied();
    let enemy = pos.board().by_color(!us);
    let empty = !occupied;

    pawns.for_each(|pawn| {
        let Some(pin_ray) = pin_info.ray_of(pawn) else {
            return;
        };

        let restricted_pawn_attacks = pin_ray & lookup::pawn_attacks(us, pawn).to_bb();
        restricted_pawn_attacks.for_each(|attk| {
            let maybe_enemy = pos.board().peek(attk);
            if let Some(enemy_piece) = maybe_enemy
                && enemy_piece.color() == !us
            {
                // capture promotion
                if attk.rank() == Rank::First || attk.rank() == Rank::Eighth {
                    moves.push(Move::capture_promotion(
                        pawn,
                        attk,
                        enemy_piece.role(),
                        Role::Queen,
                    ));
                    moves.push(Move::capture_promotion(
                        pawn,
                        attk,
                        enemy_piece.role(),
                        Role::Rook,
                    ));
                    moves.push(Move::capture_promotion(
                        pawn,
                        attk,
                        enemy_piece.role(),
                        Role::Bishop,
                    ));
                    moves.push(Move::capture_promotion(
                        pawn,
                        attk,
                        enemy_piece.role(),
                        Role::Knight,
                    ));
                } else {
                    moves.push(Move::capture(Role::Pawn, pawn, attk, enemy_piece.role()));
                }
            }
        });

        let restricted_pawn_pushes = pin_ray & lookup::pawn_pushes(us, pawn).to_bb();
        restricted_pawn_pushes.for_each(|push| {
            let not_occupied = !pos.board().occupied().is_square_set(push);
            if not_occupied {
                if push.rank() == Rank::First || push.rank() == Rank::Eighth {
                    moves.push(Move::promotion(pawn, push, Role::Queen));
                    moves.push(Move::promotion(pawn, push, Role::Rook));
                    moves.push(Move::promotion(pawn, push, Role::Bishop));
                    moves.push(Move::promotion(pawn, push, Role::Knight));
                } else {
                    moves.push(Move::quiet(Role::Pawn, pawn, push));
                }
            }
        });

        let restricted_double_pushes = pin_ray & lookup::pawn_double_pushes(us, pawn).to_bb();
        restricted_double_pushes.for_each(|double_push| {
            let not_occupied = !pos.board().occupied().is_square_set(double_push);
            let mid_not_occupied = !pos.board().occupied().is_square_set(if us == Color::White {
                double_push.offset_checked(-8)
            } else {
                double_push.offset_checked(8)
            });

            if not_occupied && mid_not_occupied {
                moves.push(Move::quiet(Role::Pawn, pawn, double_push));
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
            moves.push(Move::quiet(Role::Bishop, from, to));
        });

        capture.for_each(|to| {
            moves.push(Move::capture(
                Role::Bishop,
                from,
                to,
                pos.board().peek_role_checked(to),
            ));
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
            moves.push(Move::quiet(Role::Rook, from, to));
        });

        capture.for_each(|to| {
            moves.push(Move::capture(
                Role::Rook,
                from,
                to,
                pos.board().peek_role_checked(to),
            ));
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
            moves.push(Move::quiet(Role::Queen, from, to));
        });

        capture.for_each(|to| {
            moves.push(Move::capture(
                Role::Queen,
                from,
                to,
                pos.board().peek_role_checked(to),
            ));
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn gen_unpinned_quiet_and_capture_moves(
    pos: &Position,
    us: Color,
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    moves: &mut MoveList,
) {
    let friendly = pos.board().by_color(us);
    let enemy = pos.board().by_color(!us);
    let occupied = pos.board().occupied();
    let empty = !occupied;

    let (push_dir, cap_left_dir, cap_right_dir, double_push_rank, prom_rank) = match us {
        Color::White => (
            Direction::North,
            Direction::NorthWest,
            Direction::NorthEast,
            RANK_2,
            RANK_8,
        ),
        Color::Black => (
            Direction::South,
            Direction::SouthWest,
            Direction::SouthEast,
            RANK_7,
            RANK_1,
        ),
    };

    let single = pawns.shift_dir(push_dir) & empty;
    let quiet = single & !prom_rank;
    quiet.for_each(|from| {
        moves.push(Move::quiet(
            Role::Pawn,
            from.offset_checked(push_dir.invert().offset()),
            from,
        ));
    });

    let double =
        ((pawns & double_push_rank).shift_dir(push_dir) & empty).shift_dir(push_dir) & empty;

    double.for_each(|from| {
        moves.push(Move::quiet(
            Role::Pawn,
            from.offset_checked(push_dir.invert().offset() * 2),
            from,
        ));
    });

    let cap_left = pawns.shift_dir(cap_left_dir) & enemy;
    let cap_right = pawns.shift_dir(cap_right_dir) & enemy;

    let no_prom_cap_left = cap_left & !prom_rank;
    no_prom_cap_left.for_each(|cap| {
        let enemy_role = pos.board().peek_role_checked(cap);
        moves.push(Move::capture(
            Role::Pawn,
            cap.offset_checked(cap_left_dir.invert().offset()),
            cap,
            enemy_role,
        ));
    });

    let no_prom_cap_right = cap_right & !prom_rank;
    no_prom_cap_right.for_each(|cap| {
        let enemy_role = pos.board().peek_role_checked(cap);
        moves.push(Move::capture(
            Role::Pawn,
            cap.offset_checked(cap_right_dir.invert().offset()),
            cap,
            enemy_role,
        ));
    });

    let promo_push = single & prom_rank;
    promo_push.for_each(|promo| {
        moves.push(Move::promotion(
            promo.offset_checked(push_dir.invert().offset()),
            promo,
            Role::Queen,
        ));

        moves.push(Move::promotion(
            promo.offset_checked(push_dir.invert().offset()),
            promo,
            Role::Rook,
        ));

        moves.push(Move::promotion(
            promo.offset_checked(push_dir.invert().offset()),
            promo,
            Role::Bishop,
        ));

        moves.push(Move::promotion(
            promo.offset_checked(push_dir.invert().offset()),
            promo,
            Role::Knight,
        ));
    });

    let cap_left_promo = cap_left & prom_rank;
    cap_left_promo.for_each(|cap_promo| {
        let enemy_role = pos.board().peek_role_checked(cap_promo);

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_left_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Queen,
        ));

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_left_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Rook,
        ));

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_left_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Bishop,
        ));

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_left_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Knight,
        ));
    });

    let cap_right_promo = cap_right & prom_rank;
    cap_right_promo.for_each(|cap_promo| {
        let enemy_role = pos.board().peek_role_checked(cap_promo);

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_right_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Queen,
        ));

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_right_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Rook,
        ));

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_right_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Bishop,
        ));

        moves.push(Move::capture_promotion(
            cap_promo.offset_checked(cap_right_dir.invert().offset()),
            cap_promo,
            enemy_role,
            Role::Knight,
        ));
    });

    knights.for_each(|from| {
        let attacks = lookup::knight_attacks(from).to_bb();

        let quiet = attacks & !friendly & !enemy;
        quiet.for_each(|to| {
            moves.push(Move::quiet(Role::Knight, from, to));
        });

        let captures = attacks & enemy;
        captures.for_each(|to| {
            let enemy_role = pos.board().peek_role_checked(to);
            moves.push(Move::capture(Role::Knight, from, to, enemy_role));
        });
    });

    bishops.for_each(|from| {
        let attacks = lookup::bishop_attacks(from, occupied.as_u64()).to_bb();
        let targets = attacks & !friendly;

        let captures = targets & enemy;
        let quiet = targets & empty;

        quiet.for_each(|to| {
            moves.push(Move::quiet(Role::Bishop, from, to));
        });

        captures.for_each(|to| {
            let enemy_role = pos.board().peek_role_checked(to);
            moves.push(Move::capture(Role::Bishop, from, to, enemy_role));
        });
    });

    rooks.for_each(|from| {
        let attacks = lookup::rook_attacks(from, occupied.as_u64()).to_bb();
        let targets = attacks & !friendly;

        let captures = targets & enemy;
        let quiet = targets & empty;

        quiet.for_each(|to| {
            moves.push(Move::quiet(Role::Rook, from, to));
        });

        captures.for_each(|to| {
            let enemy_role = pos.board().peek_role_checked(to);
            moves.push(Move::capture(Role::Rook, from, to, enemy_role));
        });
    });

    queens.for_each(|from| {
        let attacks = lookup::queen_attacks(from, occupied.as_u64()).to_bb();
        let targets = attacks & !friendly;

        let captures = targets & enemy;
        let quiet = targets & empty;

        quiet.for_each(|to| {
            moves.push(Move::quiet(Role::Queen, from, to));
        });

        captures.for_each(|to| {
            let enemy_role = pos.board().peek_role_checked(to);
            moves.push(Move::capture(Role::Queen, from, to, enemy_role));
        });
    });
}

fn gen_castling_moves(pos: &Position, us: Color, moves: &mut MoveList) {
    let enemy = !us;
    let board = pos.board();

    let king_sqr = board.the_king(us);

    let occupied = board.occupied();

    if pos.castling_rights().short(us) {
        let f = Square::from_u32_checked(king_sqr.as_u32() + 1);
        let g = Square::from_u32_checked(king_sqr.as_u32() + 2);
        let rook = Square::from_u32_checked(king_sqr.as_u32() + 3);

        let path = f.to_bb().set_square(g);

        if !occupied.intersects(path)
            && !board.is_square_attacked_by(f, enemy)
            && !board.is_square_attacked_by(g, enemy)
        {
            moves.push(Move::Castling {
                king: king_sqr,
                rook,
            });
        }
    }

    if pos.castling_rights().long(us) {
        let d = Square::from_u32_checked(king_sqr.as_u32() - 1);
        let c = Square::from_u32_checked(king_sqr.as_u32() - 2);
        let b = Square::from_u32_checked(king_sqr.as_u32() - 3);
        let rook = Square::from_u32_checked(king_sqr.as_u32() - 4);

        let clear = d.to_bb().set_square(c).set_square(b);

        if !occupied.intersects(clear)
            && !board.is_square_attacked_by(d, enemy)
            && !board.is_square_attacked_by(c, enemy)
        {
            moves.push(Move::Castling {
                king: king_sqr,
                rook,
            });
        }
    }
}

pub fn gen_ep_moves(pos: &Position, us: Color, moves: &mut MoveList) {
    let Some(ep) = pos.ep_square() else {
        return;
    };

    let enemy = !us;
    let king_sqr = pos.board().the_king(us);
    let pawns = pos.board().pawns(us);

    let enemy_rooks = pos.board().rooks(enemy);
    let enemy_bishops = pos.board().bishops(enemy);
    let enemy_queens = pos.board().queens(enemy);

    pawns.for_each(|from| {
        if lookup::pawn_attacks(us, from) & (1 << ep.as_u32()) == 0 {
            return;
        }

        let captured = Square::of(ep.file(), from.rank());

        if pos.board().peek(captured) != Some(Piece::of(Role::Pawn, enemy)) {
            return;
        }

        let legal = if (enemy_bishops | enemy_rooks | enemy_queens).empty() {
            true
        } else {
            let occupied = pos
                .board()
                .occupied()
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
            moves.push(Move::EnPassant { from, to: ep });
        }
    });
}

fn gen_evasions(
    pos: &Position,
    us: Color,
    king_sqr: Square,
    checker: Bitboard,
    pinned: Bitboard,
    moves: &mut MoveList,
) {
    gen_king_moves(pos, us, king_sqr, moves);

    // although we already know the existence of checker due to call site
    // let-else is more performant than panicking alternative
    let Some(checker) = checker.only_first_square() else {
        return;
    };

    let evasion_mask = lookup::ray_between(king_sqr, checker).to_bb();

    gen_blocking_moves(pos, us, evasion_mask, checker, pinned, moves);
    gen_ep_evasions(pos, us, checker, pinned, moves);
}

// NOTE consider calculating via enemy piece attack maps in order to avoid branches
fn gen_king_moves(pos: &Position, us: Color, king_sqr: Square, moves: &mut MoveList) {
    let board = pos.board();

    let friendly = board.by_color(us);
    let enemy = board.by_color(!us);

    let attacks = lookup::king_attacks(king_sqr).to_bb();

    let quiet = attacks & !friendly & !enemy;
    quiet.for_each(|to| {
        if king_move_is_safe(pos, us, king_sqr, to) {
            moves.push(Move::quiet(Role::King, king_sqr, to));
        }
    });

    let captures = attacks & enemy;
    captures.for_each(|to| {
        if king_move_is_safe(pos, us, king_sqr, to) {
            moves.push(Move::capture(
                Role::King,
                king_sqr,
                to,
                board.peek_role_checked(to),
            ));
        }
    });
}

fn gen_blocking_moves(
    pos: &Position,
    us: Color,
    evasion_mask: Bitboard,
    checker: Square,
    pinned: Bitboard,
    moves: &mut MoveList,
) {
    let board = pos.board();
    let occupied = board.occupied();

    let rooks = board.rooks(us) & !pinned;
    let bishops = board.bishops(us) & !pinned;
    let queens = board.queens(us) & !pinned;
    let knights = board.knights(us) & !pinned;
    let pawns = board.pawns(us) & !pinned;

    let capture_mask = checker.to_bb();

    let checker_role = pos.board().peek_role_checked(checker);

    pawns.for_each(|pawn| {
        let pawn_pushes = lookup::pawn_pushes(us, pawn).to_bb() & evasion_mask;
        pawn_pushes.for_each(|push| {
            let promotion_rank = push.rank() == Rank::First || push.rank() == Rank::Eighth;
            if promotion_rank {
                for role in [Role::Queen, Role::Rook, Role::Bishop, Role::Knight] {
                    moves.push(Move::promotion(pawn, push, role));
                }
            } else {
                moves.push(Move::quiet(Role::Pawn, pawn, push));
            }
        });

        let pawn_double_pushes = lookup::pawn_double_pushes(us, pawn).to_bb() & evasion_mask;
        pawn_double_pushes.for_each(|push| {
            if occupied.is_square_set(push.offset_checked(if us == Color::White { -8 } else { 8 }))
            {
                return;
            }

            moves.push(Move::quiet(Role::Pawn, pawn, push));
        });

        let pawn_attacks = lookup::pawn_attacks(us, pawn).to_bb() & checker.to_bb();
        pawn_attacks.for_each(|_| {
            let promotion_rank = checker.rank() == Rank::First || checker.rank() == Rank::Eighth;

            if promotion_rank {
                for role in [Role::Queen, Role::Rook, Role::Bishop, Role::Knight] {
                    moves.push(Move::capture_promotion(pawn, checker, checker_role, role));
                }
            } else {
                moves.push(Move::capture(Role::Pawn, pawn, checker, checker_role));
            }
        });
    });

    let ray_mask = capture_mask | evasion_mask;
    ray_mask.for_each(|attk| {
        let queen_attackers = lookup::queen_attacks(attk, occupied.as_u64()).to_bb() & queens;
        queen_attackers.for_each(|queen| {
            if attk == checker {
                moves.push(Move::capture(Role::Queen, queen, attk, checker_role));
            } else {
                moves.push(Move::quiet(Role::Queen, queen, attk));
            }
        });

        let rook_atackers = lookup::rook_attacks(attk, occupied.as_u64()).to_bb() & rooks;
        rook_atackers.for_each(|rook| {
            if attk == checker {
                moves.push(Move::capture(Role::Rook, rook, attk, checker_role));
            } else {
                moves.push(Move::quiet(Role::Rook, rook, attk));
            }
        });

        let bishop_attackers = lookup::bishop_attacks(attk, occupied.as_u64()).to_bb() & bishops;
        bishop_attackers.for_each(|bishop| {
            if attk == checker {
                moves.push(Move::capture(Role::Bishop, bishop, attk, checker_role));
            } else {
                moves.push(Move::quiet(Role::Bishop, bishop, attk));
            }
        });

        let knight_attackers = lookup::knight_attacks(attk).to_bb() & knights;
        knight_attackers.for_each(|knight| {
            if attk == checker {
                moves.push(Move::capture(Role::Knight, knight, attk, checker_role));
            } else {
                moves.push(Move::quiet(Role::Knight, knight, attk));
            }
        });
    });
}

fn gen_ep_evasions(
    pos: &Position,
    us: Color,
    checker: Square,
    pinned: Bitboard,
    moves: &mut MoveList,
) {
    let Some(ep_sqr) = pos.ep_square() else {
        return;
    };

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

    let pawns = pos.board().pawns(us) & lookup::pawn_attacks(!us, ep_sqr).to_bb();
    let king_sqr = pos.board().the_king(us);

    pawns.for_each(|pawn| {
        if pinned.is_square_set(pawn) {
            return;
        }

        let enemy_sliders = pos.board().rooks(!us) | pos.board().queens(!us);

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
            moves.push(Move::EnPassant {
                from: pawn,
                to: ep_sqr,
            });
        }
    });
}

fn king_move_is_safe(pos: &Position, us: Color, from: Square, to: Square) -> bool {
    let enemy = !us;
    let board = pos.board();

    let occupied = pos.board().occupied().clear_square(from).set_square(to);

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
        pinned_pieces: Bitboard,
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
            if self.pinned_pieces.is_square_set(square) {
                // SAFETY: calling this unsafe function is sound because square is pinned
                Some(unsafe { self.ray_of_unchecked(square) })
            } else {
                None
            }
        }

        pub fn pinned(&self) -> Bitboard {
            self.pinned_pieces
        }

        pub fn compute(pos: &Position, us: Color) -> Self {
            let them = !us;

            let board = pos.board();
            let occupied = board.occupied();
            let king = board.the_king(us);

            let enemy_bishops = board.bishops(them) | board.queens(them);
            let enemy_rooks = board.rooks(them) | board.queens(them);

            let mut pinned_pieces = 0_u64.to_bb();
            let mut pin_rays = [MaybeUninit::<Bitboard>::uninit(); 8];
            let mut squares = [MaybeUninit::<Square>::uninit(); 8];
            let mut len = 0;

            let diagonal_attacks = lookup::diagonal_rays_from(king).to_bb() & enemy_bishops;
            diagonal_attacks.for_each(|attacker| {
                do_compute(
                    occupied,
                    king,
                    &mut pinned_pieces,
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
                    &mut pinned_pieces,
                    &mut pin_rays,
                    &mut squares,
                    &mut len,
                    attacker,
                );
            });

            Self {
                pinned_pieces,
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
