use arrayvec::ArrayVec;
use types::{
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

use crate::chessboard::ChessBoard;

pub fn gen_legal_moves(pos: &ChessBoard) -> ArrayVec<Move, 218> {
    let mut moves = ArrayVec::<Move, 218>::new();
    let king_checkers = pos.checkers(pos.turn());
    let king_sqr = pos.board().the_king(pos.turn());

    // NOTE an array of roles and corresponding squares
    // NOTE consider using lighter representation for Move
    let (pinned, pin_rays) = compute_pinned_pieces_of(pos);

    if king_checkers.empty() {
        gen_standart_moves(pos, pinned, &pin_rays, &mut moves);
        gen_castling_moves(pos, &mut moves);
        gen_ep_moves(pos, &mut moves);
    } else if king_checkers.popcnt() == 2 {
        gen_king_moves(pos, king_sqr, &mut moves);
    } else {
        gen_evasions(pos, king_sqr, king_checkers, pinned, &mut moves);
    }

    moves
}

#[rustfmt::skip]
fn gen_standart_moves(pos: &ChessBoard, pinned: Bitboard, pin_rays: &[Bitboard; 64], moves: &mut ArrayVec<Move, 218>) {
    let our = pos.turn();
    let board = pos.board();

    let unpinned = board.by_color(our) & !pinned;

    let pawns = board.pawns(our);
    let knights = board.knights(our);
    let bishops = board.bishops(our);
    let rooks = board.rooks(our);
    let queens = board.queens(our);

    gen_king_moves(pos, board.the_king(our), moves);

    gen_unpinned_standart_moves(
        pos,
        our,
        pawns   & unpinned,
        knights & unpinned,
        bishops & unpinned,
        rooks   & unpinned,
        queens  & unpinned,
        moves,
    );

    gen_pinned_standart_moves(
        pos,
        our,
        pawns   & pinned,
        bishops & pinned,
        rooks   & pinned,
        queens  & pinned,
        pin_rays,
        moves,
    );
}

fn gen_castling_moves(pos: &ChessBoard, moves: &mut ArrayVec<Move, 218>) {
    let our = pos.turn();
    let enemy = !our;
    let board = pos.board();

    let king_sqr = board.the_king(our);

    let occupied = board.occupied();

    if pos.castling_rights().short(our) {
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
    if pos.castling_rights().long(our) {
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

pub fn gen_ep_moves(pos: &ChessBoard, moves: &mut ArrayVec<Move, 218>) {
    let Some(ep) = pos.ep_square() else {
        return;
    };

    let our = pos.turn();
    let enemy = !our;
    let king_sqr = pos.board().the_king(our);
    let pawns = pos.board().pawns(our);

    let enemy_rooks = pos.board().rooks(enemy);
    let enemy_bishops = pos.board().bishops(enemy);
    let enemy_queens = pos.board().queens(enemy);

    pawns.for_each(|from| {
        if lookup::pawn_attacks(our, from) & (1 << ep.as_u32()) == 0 {
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
    pos: &ChessBoard,
    king_sqr: Square,
    checker: Bitboard,
    pinned: Bitboard,
    moves: &mut ArrayVec<Move, 218>,
) {
    gen_king_moves(pos, king_sqr, moves);

    let Some(checker) = checker.only_first_square() else {
        return;
    };

    let evasion_mask = lookup::ray_between(king_sqr, checker).to_bb();

    gen_blocking_moves(pos, evasion_mask, checker, pinned, moves);
    gen_ep_evasions(pos, checker, pinned, moves);
}

// NOTE consider calculating via enemy piece attack maps in order to avoid branches
#[inline(always)]
fn gen_king_moves(pos: &ChessBoard, king_sqr: Square, moves: &mut ArrayVec<Move, 218>) {
    let board = pos.board();
    let our = pos.turn();

    let friendly = board.by_color(our);
    let enemy = board.by_color(!our);

    let attacks = lookup::king_attacks(king_sqr).to_bb();

    let quiet = attacks & !friendly & !enemy;
    quiet.for_each(|to| {
        if king_move_is_safe(pos, king_sqr, to) {
            moves.push(Move::quiet(Role::King, king_sqr, to));
        }
    });

    let captures = attacks & enemy;
    captures.for_each(|to| {
        if king_move_is_safe(pos, king_sqr, to) {
            moves.push(Move::capture(
                Role::King,
                king_sqr,
                to,
                board.peek_role_checked(to),
            ));
        }
    });
}

fn king_move_is_safe(pos: &ChessBoard, from: Square, to: Square) -> bool {
    let our = pos.turn();
    let enemy = !our;
    let board = pos.board();

    let occupied = pos.board().occupied().clear_square(from).set_square(to);

    let queens = board.queens(enemy);
    let pawns = board.pawns(enemy);
    let knights = board.knights(enemy);
    let bishops = board.bishops(enemy) | queens;
    let rooks = board.rooks(enemy) | queens;
    let king = board.king(enemy);

    (lookup::pawn_attacks(our, to).to_bb() & pawns).empty()
        && (lookup::knight_attacks(to).to_bb() & knights).empty()
        && (lookup::bishop_attacks(to, occupied.as_u64()).to_bb() & bishops).empty()
        && (lookup::rook_attacks(to, occupied.as_u64()).to_bb() & rooks).empty()
        && (lookup::king_attacks(to).to_bb() & king).empty()
}

fn gen_blocking_moves(
    pos: &ChessBoard,
    evasion_mask: Bitboard,
    checker: Square,
    pinned: Bitboard,
    moves: &mut ArrayVec<Move, 218>,
) {
    let our = pos.turn();
    let board = pos.board();
    let occupied = board.occupied();

    let rooks = board.rooks(our) & !pinned;
    let bishops = board.bishops(our) & !pinned;
    let queens = board.queens(our) & !pinned;
    let knights = board.knights(our) & !pinned;
    let pawns = board.pawns(our) & !pinned;

    let capture_mask = checker.to_bb();

    let checker_role = pos.board().peek_role_checked(checker);

    pawns.for_each(|pawn| {
        let pawn_pushes = lookup::pawn_pushes(our, pawn).to_bb() & evasion_mask;
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

        let pawn_double_pushes = lookup::pawn_double_pushes(our, pawn).to_bb() & evasion_mask;
        pawn_double_pushes.for_each(|push| {
            if occupied.is_square_set(push.offset_checked(if our == Color::White { -8 } else { 8 }))
            {
                return;
            }

            moves.push(Move::quiet(Role::Pawn, pawn, push));
        });

        let pawn_attacks = lookup::pawn_attacks(our, pawn).to_bb() & checker.to_bb();
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
    pos: &ChessBoard,
    checker: Square,
    pinned: Bitboard,
    moves: &mut ArrayVec<Move, 218>,
) {
    let Some(ep_sqr) = pos.ep_square() else {
        return;
    };

    let target_pawn = Square::of(
        ep_sqr.file(),
        if pos.turn() == Color::White {
            Rank::Fifth
        } else {
            Rank::Fourth
        },
    );

    if target_pawn != checker {
        return;
    }

    let pawns = pos.board().pawns(pos.turn()) & lookup::pawn_attacks(!pos.turn(), ep_sqr).to_bb();
    let king_sqr = pos.board().the_king(pos.turn());

    pawns.for_each(|pawn| {
        if pinned.is_square_set(pawn) {
            return;
        }

        let enemy_sliders = pos.board().rooks(!pos.turn()) | pos.board().queens(!pos.turn());

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

#[allow(clippy::too_many_arguments)]
fn gen_unpinned_standart_moves(
    pos: &ChessBoard,
    side: Color,
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    moves: &mut ArrayVec<Move, 218>,
) {
    let friendly = pos.board().by_color(side);
    let enemy = pos.board().by_color(!side);
    let occupied = pos.board().occupied();
    let empty = !occupied;

    let (push_dir, cap_left_dir, cap_right_dir, double_push_rank, prom_rank) = match pos.turn() {
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

fn gen_pinned_standart_moves(
    pos: &ChessBoard,
    side: Color,
    pawns: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    pin_rays: &[Bitboard; 64],
    moves: &mut ArrayVec<Move, 218>,
) {
    let enemy = pos.board().by_color(!side);
    let occupied = pos.board().occupied();
    let empty = !occupied;

    pawns.for_each(|pawn| {
        let pin_ray = pin_rays[pawn.as_usize()];

        let restricted_pawn_attacks = pin_ray & lookup::pawn_attacks(side, pawn).to_bb();
        restricted_pawn_attacks.for_each(|attk| {
            let maybe_enemy = pos.board().peek(attk);
            if let Some(enemy_piece) = maybe_enemy
                && enemy_piece.color() == !side
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

        let restricted_pawn_pushes = pin_ray & lookup::pawn_pushes(side, pawn).to_bb();
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

        let restricted_double_pushes = pin_ray & lookup::pawn_double_pushes(side, pawn).to_bb();
        restricted_double_pushes.for_each(|double_push| {
            let not_occupied = !pos.board().occupied().is_square_set(double_push);
            let mid_not_occupied = !pos
                .board()
                .occupied()
                .is_square_set(if side == Color::White {
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
        let pin_ray = pin_rays[from.as_usize()];
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
        let pin_ray = pin_rays[from.as_usize()];
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
        let pin_ray = pin_rays[from.as_usize()];
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

#[inline(always)]
pub fn compute_pinned_pieces_of(pos: &ChessBoard) -> (Bitboard, [Bitboard; 64]) {
    let mut pinned = Bitboard::new_empty();
    let mut pin_rays = [u64::MAX.to_bb(); 64];

    let us = pos.turn();
    let them = !us;

    let board = pos.board();

    let pieces = board.by_color(us);

    let occupied = board.occupied();

    let king = board.the_king(us);

    let enemy_rooks = board.rooks(them) | board.queens(them);
    let enemy_bishops = board.bishops(them) | board.queens(them);

    compute_pinned(
        &mut pinned,
        &mut pin_rays,
        pieces,
        occupied,
        lookup::diagonal_rays_from(king).to_bb() & enemy_bishops,
        king,
    );

    compute_pinned(
        &mut pinned,
        &mut pin_rays,
        pieces,
        occupied,
        lookup::orthogonal_rays_from(king).to_bb() & enemy_rooks,
        king,
    );

    (pinned, pin_rays)
}

fn compute_pinned(
    pinned: &mut Bitboard,
    pin_rays: &mut [Bitboard; 64],
    us: Bitboard,
    occupied: Bitboard,
    xray: Bitboard,
    king: Square,
) {
    xray.for_each(|attacker| {
        let between = lookup::ray_between(king, attacker).to_bb();
        let between_inclusive = between | king.to_bb() | attacker.to_bb();
        let blockers = (between & occupied) | (us & between);

        if let Some(blocker_sqr) = blockers.only_first_square() {
            *pinned = pinned.set_square(blocker_sqr);
            pin_rays[blocker_sqr.as_usize()] = between_inclusive;
        }
    });
}

mod pin_info {
    use std::mem::MaybeUninit;

    use types::{
        bitboard::{Bitboard, ToBitboard},
        square::Square,
    };

    use crate::chessboard::ChessBoard;

    struct PinInfo {
        pinned_pieces: Bitboard,
        pin_rays: [MaybeUninit<Bitboard>; 64],
    }

    impl PinInfo {
        fn new() -> Self {
            Self {
                pinned_pieces: 0_u64.to_bb(),
                pin_rays: [MaybeUninit::uninit(); 64],
            }
        }

        fn push(&mut self, square: Square, ray: Bitboard) {
            // SAFETY: self.len is always less than 64
            unsafe {
                *self.pin_rays.get_unchecked_mut(square.as_usize()) = MaybeUninit::new(ray);
            }

            self.pinned_pieces = self.pinned_pieces.set_square(square);
        }

        pub fn ray_of(&self, square: Square) -> Option<Bitboard> {
            if self.pinned_pieces.is_square_set(square) {
                return unsafe {
                    Some(self.pin_rays.get_unchecked(square.as_usize()).assume_init())
                };
            }

            None
        }

        pub fn compute(pos: &ChessBoard) -> Self {
            let us = pos.turn();
            let them = !us;

            let board = pos.board();
            let pieces = board.by_color(us);
            let occupied = board.occupied();
            let king = board.the_king(us);

            let enemy_bishops = board.bishops(them) | board.queens(them);
            let enemy_rooks = board.rooks(them) | board.queens(them);

            let mut pinned_pieces = 0_u64.to_bb();
            let mut pin_rays = [MaybeUninit::<Bitboard>::uninit(); 64];

            let diagonal_attacks = lookup::diagonal_rays_from(king).to_bb() & enemy_rooks;
            diagonal_attacks.for_each(|attacker| {
                let between = lookup::ray_between(king, attacker).to_bb();
                let between_inclusive = between | king.to_bb() | attacker.to_bb();
                let blockers = (between & occupied) | (pieces & between);

                if let Some(blocker_sqr) = blockers.only_first_square() {
                    pinned_pieces = pinned_pieces.set_square(blocker_sqr);
                    unsafe {
                        *pin_rays.get_unchecked_mut(blocker_sqr.as_usize()) =
                            MaybeUninit::new(between_inclusive)
                    }
                }
            });

            let orthogonal_attacks = lookup::orthogonal_rays_from(king).to_bb() & enemy_bishops;
            orthogonal_attacks.for_each(|attacker| {
                let between = lookup::ray_between(king, attacker).to_bb();
                let between_inclusive = between | king.to_bb() | attacker.to_bb();
                let blockers = (between & occupied) | (pieces & between);

                if let Some(blocker_sqr) = blockers.only_first_square() {
                    pinned_pieces = pinned_pieces.set_square(blocker_sqr);
                    unsafe {
                        *pin_rays.get_unchecked_mut(blocker_sqr.as_usize()) =
                            MaybeUninit::new(between_inclusive)
                    }
                }
            });

            Self {
                pinned_pieces,
                pin_rays: pin_rays,
            }
        }
    }
}
