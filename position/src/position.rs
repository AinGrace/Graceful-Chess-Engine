use std::{
    error::Error,
    fmt::{Debug, Display},
    hint::unreachable_unchecked,
    mem,
    num::NonZeroU8,
};

use lookup::{PSQT, phase_weight};
use types::{
    MoveList,
    bitboard::Bitboard,
    castlings::Castlings,
    chess_move::{Move, MoveFlag},
    color::Color,
    piece::Piece,
    rank::Rank,
    role::Role,
    square::Square,
};

use crate::{
    board::Board,
    fen::Fen,
    move_gen,
    zobrist::{self, z_black_to_move, z_castling, z_ep, z_key},
};

#[derive(Debug)]
pub struct InvalidMoveError {
    mv: String,
    pos: Position,
}

impl Error for InvalidMoveError {}

impl Display for InvalidMoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl InvalidMoveError {
    pub fn mv(&self) -> &str {
        &self.mv
    }

    pub fn chessboard_ref(&self) -> &Position {
        &self.pos
    }

    pub fn chessboard(self) -> Position {
        self.pos
    }
}

#[derive(Debug)]
pub enum PositionError {
    TooManyKings,
    TooMuchMaterial,
    InvalidCastling,
    InvalidEnPassaunt,
    NoLegalMoves,
    InvalidPawn,
}

impl Error for PositionError {}

impl Display for PositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undo {
    m: Move,
    castlings: Castlings,
    ep: Option<Square>,
    half_moves: u8,
    full_moves: NonZeroU8,
    z_hash: u64,
    captured_piece: Option<Piece>,
}

#[derive(Clone)]
pub struct Position {
    board: Board,
    turn: Color,
    castlings: Castlings,
    ep: Option<Square>,
    half_moves: u8,
    full_moves: NonZeroU8,
    phase: u8,
    mg_score: i16,
    eg_score: i16,
    z_hash: u64,
}

impl Position {
    /// create a new chessboard with standart position
    pub fn new() -> Self {
        let mut pos = Self {
            board: Board::new(),
            turn: Color::White,
            castlings: Castlings::new(),
            ep: None,
            half_moves: 0,
            full_moves: NonZeroU8::MIN,
            mg_score: 0,
            eg_score: 0,
            phase: 0,
            z_hash: 0, // temporary value
        };

        let phase: u8 = pos
            .board
            .mailbox()
            .iter()
            .flatten()
            .map(|p| phase_weight(p.role()))
            .sum();

        let mut mg_score = 0;
        let mut eg_score = 0;

        for i in 0..pos.board.mailbox().len() {
            if let Some(piece) = pos.board.mailbox()[i] {
                let sq = if piece.color() == Color::White {
                    i
                } else {
                    i ^ 56 // mirror rank for Black
                };

                let sign = if piece.color() == Color::White { 1 } else { -1 };

                mg_score += sign * PSQT.mg[piece.role().as_usize()][sq];
                eg_score += sign * PSQT.eg[piece.role().as_usize()][sq];
            }
        }

        let z_hash = zobrist::compute_hash(&pos);

        pos.z_hash = z_hash;
        pos.phase = phase;
        pos.mg_score = mg_score;
        pos.eg_score = eg_score;
        pos
    }

    /// create a new chessboard from Fen struct
    pub fn from_fen(
        Fen {
            board,
            turn,
            castlings,
            ep_square,
            half_moves,
            full_moves,
            ..
        }: Fen,
    ) -> Result<Self, PositionError> {
        let mut pos = Self {
            board,
            castlings,
            turn,
            ep: ep_square,
            half_moves,
            full_moves,
            mg_score: 0,
            eg_score: 0,
            phase: 0,
            z_hash: 0, //temporary value
        };

        pos.health_check()?;

        let phase: u8 = pos
            .board
            .mailbox()
            .iter()
            .flatten()
            .map(|p| phase_weight(p.role()))
            .sum();

        let mut mg_score = 0;
        let mut eg_score = 0;

        for i in 0..pos.board.mailbox().len() {
            if let Some(piece) = pos.board.mailbox()[i] {
                let sq = if piece.color() == Color::White {
                    i
                } else {
                    i ^ 56 // mirror rank for Black
                };

                let sign = if piece.color() == Color::White { 1 } else { -1 };

                mg_score += sign * PSQT.mg[piece.role().as_usize()][sq];
                eg_score += sign * PSQT.eg[piece.role().as_usize()][sq];
            }
        }

        let z_hash = zobrist::compute_hash(&pos);
        pos.z_hash = z_hash;
        pos.mg_score = mg_score;
        pos.eg_score = eg_score;
        pos.phase = phase;

        Ok(pos)
    }

    /// Returns a bitboard of pieces giving check to the king
    #[inline(always)]
    #[track_caller]
    pub fn checkers_to(&self, side: Color) -> Bitboard {
        let king = self.board.the_king(side);
        self.board.attacks_to(king, !side)
    }

    pub fn in_check(&self) -> bool {
        self.checkers_to(self.turn()).present()
    }

    pub fn is_checkmate(&self) -> bool {
        self.checkers_to(self.turn).present() && self.legal_moves().is_empty()
    }

    pub fn is_stalemate(&self) -> bool {
        self.checkers_to(self.turn).empty() && self.legal_moves().is_empty()
    }

    pub fn half_moves(&self) -> u8 {
        self.half_moves
    }

    pub fn full_moves(&self) -> u8 {
        self.full_moves.get()
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.z_hash
    }

    /// Generate and return a list of legal moves for the curent position
    pub fn legal_moves(&self) -> MoveList {
        move_gen::gen_legal_moves_for_v2::<false>(self, self.turn())
    }

    pub fn legal_moves_for(&self, us: Color) -> MoveList {
        move_gen::gen_legal_moves_for_v2::<false>(self, us)
    }

    pub fn legal_captures(&self) -> MoveList {
        move_gen::gen_legal_moves_for_v2::<true>(self, self.turn())
    }

    pub fn legal_captures_for(&self, us: Color) -> MoveList {
        move_gen::gen_legal_moves_for_v2::<true>(self, us)
    }

    #[inline(always)]
    pub const fn turn(&self) -> Color {
        self.turn
    }

    #[inline(always)]
    pub fn board(&self) -> &Board {
        &self.board
    }

    #[inline(always)]
    pub fn board_owned(&self) -> Board {
        self.board.clone()
    }

    pub fn ep_square(&self) -> Option<Square> {
        self.ep
    }

    pub fn castling_rights(&self) -> &Castlings {
        &self.castlings
    }

    pub fn tapered_score(&self) -> i16 {
        let phase = self.phase.clamp(0, 24) as i16;
        (self.mg_score * phase + self.eg_score * (24 - phase)) / 24
    }

    /// Checks move for legality and then executes it
    pub fn do_move(&mut self, mv: Move) -> Result<Undo, InvalidMoveError> {
        if self.is_legal_move(mv) {
            // SAFERY: mv is checked to be a valid move
            Ok(unsafe { self.do_move_unchecked(mv) })
        } else {
            // TODO consider using long algebraic notation instead of uci
            Err(InvalidMoveError {
                mv: mv.to_uci(),
                pos: self.clone(),
            })
        }
    }

    /// parse uci string and execute it
    pub fn uci_move(&mut self, raw_uci: &str) -> Result<Undo, InvalidMoveError> {
        let uci_move = match self.parse_uci(raw_uci) {
            Some(uci_move) => uci_move,
            None => {
                return Err(InvalidMoveError {
                    mv: raw_uci.into(),
                    pos: self.clone(),
                });
            }
        };

        //SAFETY: uci_move is checked to be a valid move
        Ok(unsafe { self.do_move_unchecked(uci_move) })
    }

    pub fn uci_move_checked(&mut self, raw_uci: &str) -> Undo {
        let uci_move = self
            .parse_uci(raw_uci)
            .unwrap_or_else(|| panic!("invalid uci {raw_uci}"));

        //SAFETY: uci_move is checked to be a valid move
        unsafe { self.do_move_unchecked(uci_move) }
    }

    pub fn uci_moves_checked(&mut self, raw_uci_moves: &str) {
        for raw_uci in raw_uci_moves.split_ascii_whitespace() {
            let uci_move = self
                .parse_uci(raw_uci)
                .unwrap_or_else(|| panic!("invalid uci {raw_uci}"));

            //SAFETY: uci_move is checked to be a valid move
            unsafe {
                self.do_move_unchecked(uci_move);
            }
        }
    }

    pub unsafe fn undo_move(&mut self, undo: Undo) {
        self.turn = !self.turn;

        let mv = undo.m;
        let from = mv.from();
        let to = mv.to();

        match mv.flag() {
            MoveFlag::KingCastle | MoveFlag::QueenCastle => {
                let (rook_from, rook_to) = match to {
                    Square::C1 => (Square::A1, Square::D1),
                    Square::C8 => (Square::A8, Square::D8),
                    Square::G1 => (Square::H1, Square::F1),
                    Square::G8 => (Square::H8, Square::F8),

                    _ => unsafe { unreachable_unchecked() },
                };

                unsafe {
                    let king_piece = self.remove_piece(to);
                    let rook_piece = self.remove_piece(rook_to);
                    self.set_piece(king_piece, from);
                    self.set_piece(rook_piece, rook_from);
                }
            }
            MoveFlag::EnPassant => {
                // SAFETY: same as above
                let our_pawn = unsafe { self.remove_piece(to) };

                let enemy_pawn = if self.turn == Color::White {
                    Piece::BPawn
                } else {
                    Piece::WPawn
                };

                unsafe {
                    self.set_piece(our_pawn, from);
                    self.set_piece(enemy_pawn, Square::of(to.file(), from.rank()));
                }
            }

            rest => {
                // SAFETY: to is quaranteed to be valid
                let our_piece = unsafe { self.remove_piece(to) };

                if rest.is_promotion() {
                    let our_pawn = if self.turn == Color::White {
                        Piece::WPawn
                    } else {
                        Piece::BPawn
                    };
                    unsafe {
                        self.set_piece(our_pawn, from);
                    }
                } else {
                    unsafe {
                        self.set_piece(our_piece, from);
                    }
                }

                if let Some(captured) = undo.captured_piece {
                    unsafe {
                        self.set_piece(captured, to);
                    }
                }
            }
        }

        self.castlings = undo.castlings;
        self.ep = undo.ep;
        self.half_moves = undo.half_moves;
        self.full_moves = undo.full_moves;
        self.z_hash = undo.z_hash;
    }

    /// parse uci string as ChessMove enum
    pub fn parse_uci(&self, raw_uci: &str) -> Option<Move> {
        self.legal_moves()
            .into_iter()
            .find(|mv| mv.to_uci() == raw_uci)
    }

    /// Validates move for legality
    pub fn is_legal_move(&self, mv: Move) -> bool {
        let legal_moves = self.legal_moves();
        legal_moves.contains(&mv)
    }

    unsafe fn set_piece(&mut self, piece: Piece, sqr: Square) {
        self.board.set_piece_at(piece, sqr);
        self.z_hash ^= z_key(piece, sqr);
        let (sign, sqr) = match self.turn {
            Color::White => (1, sqr.as_usize()),
            Color::Black => (-1, sqr.as_usize() ^ 56),
        };

        self.mg_score += sign * PSQT.mg[piece.role().as_usize()][sqr];
        self.eg_score += sign * PSQT.eg[piece.role().as_usize()][sqr];
        self.phase -= phase_weight(piece.role());
    }

    unsafe fn remove_piece(&mut self, sqr: Square) -> Piece {
        let piece = unsafe { self.board.take_piece_at_unchecked(sqr) };
        self.z_hash ^= z_key(piece, sqr);
        let (sign, sqr) = match self.turn {
            Color::White => (1, sqr.as_usize()),
            Color::Black => (-1, sqr.as_usize() ^ 56),
        };

        self.mg_score += sign * PSQT.mg[piece.role().as_usize()][sqr];
        self.eg_score += sign * PSQT.eg[piece.role().as_usize()][sqr];
        self.phase += phase_weight(piece.role());

        piece
    }

    /// Calling this method without validity quarantees by the caller may corrupt the state of Position
    pub unsafe fn do_move_unchecked(&mut self, mv: Move) -> Undo {
        let mut undo = Undo {
            m: mv,
            castlings: self.castlings,
            half_moves: self.half_moves,
            full_moves: self.full_moves,
            z_hash: self.z_hash,
            captured_piece: None,
            ep: self.ep_square(),
        };

        let us = self.turn;

        if let Some(ep) = self.ep.take() {
            self.z_hash ^= z_ep(ep);
        }

        let from = mv.from();
        let to = mv.to();

        let moving_piece = unsafe { self.board.peek_unchecked(from) };

        match mv.flag() {
            MoveFlag::KingCastle | MoveFlag::QueenCastle => {
                let (rook_from, rook_to) = match to {
                    Square::C1 => (Square::A1, Square::D1),
                    Square::C8 => (Square::A8, Square::D8),
                    Square::G1 => (Square::H1, Square::F1),
                    Square::G8 => (Square::H8, Square::F8),

                    _ => unsafe { unreachable_unchecked() },
                };

                let king_piece = Piece::of(Role::King, us);
                let rook_piece = Piece::of(Role::Rook, us);

                unsafe { self.remove_piece(from) };
                unsafe { self.set_piece(king_piece, to) };

                unsafe { self.remove_piece(rook_from) };
                unsafe { self.set_piece(rook_piece, rook_to) };

                self.z_hash ^= z_castling(self.castlings);
                self.castlings.remove_all_of(us);
                self.z_hash ^= z_castling(self.castlings);

                self.half_moves += 1;
            }
            MoveFlag::EnPassant => {
                self.half_moves = 0;
                let captured_sqr = Square::of(to.file(), from.rank());

                unsafe { self.remove_piece(from) };
                unsafe { self.remove_piece(captured_sqr) };

                unsafe {
                    self.set_piece(moving_piece, to);
                }

                undo.captured_piece = Some(Piece::of(Role::Pawn, !us));
            }
            rest => {
                // set en_passaunt on pawn double push
                if matches!(rest, MoveFlag::DoublePush) {
                    let ep = ((from as u8 + to as u8) >> 1) as u32;
                    let ep_sqr = unsafe { Square::from_u32_unchecked(ep) };
                    self.ep = Some(ep_sqr);
                    self.z_hash ^= z_ep(ep_sqr);
                }

                unsafe { self.remove_piece(from) };

                if rest.is_capture() {
                    let captured = unsafe { self.remove_piece(to) };
                    undo.captured_piece = Some(captured);

                    if captured.role() == Role::Rook {
                        // change castling rights on capture
                        match (us, to) {
                            (Color::Black, Square::H1) => {
                                self.z_hash ^= z_castling(self.castlings);
                                self.castlings.remove_w_short();
                                self.z_hash ^= z_castling(self.castlings);
                            }
                            (Color::White, Square::H8) => {
                                self.z_hash ^= z_castling(self.castlings);
                                self.castlings.remove_b_short();
                                self.z_hash ^= z_castling(self.castlings);
                            }
                            (Color::Black, Square::A1) => {
                                self.z_hash ^= z_castling(self.castlings);
                                self.castlings.remove_w_long();
                                self.z_hash ^= z_castling(self.castlings);
                            }
                            (Color::White, Square::A8) => {
                                self.z_hash ^= z_castling(self.castlings);
                                self.castlings.remove_b_long();
                                self.z_hash ^= z_castling(self.castlings);
                            }

                            _rest => (),
                        }
                    }
                }

                // if promotion exists set it at destination square, otherwise set the moving piece
                let piece = match rest {
                    MoveFlag::PromoN | MoveFlag::PromoCapN => Piece::of(Role::Knight, us),
                    MoveFlag::PromoB | MoveFlag::PromoCapB => Piece::of(Role::Bishop, us),
                    MoveFlag::PromoR | MoveFlag::PromoCapR => Piece::of(Role::Rook, us),
                    MoveFlag::PromoQ | MoveFlag::PromoCapQ => Piece::of(Role::Queen, us),
                    _ => moving_piece,
                };

                unsafe { self.set_piece(piece, to) };

                // change castling rights on quiet move
                match (us, moving_piece.role(), from) {
                    (Color::White, Role::King, Square::E1) => {
                        self.z_hash ^= z_castling(self.castlings);
                        self.castlings.remove_white();
                        self.z_hash ^= z_castling(self.castlings);
                    }
                    (Color::White, Role::Rook, Square::A1) => {
                        self.z_hash ^= z_castling(self.castlings);
                        self.castlings.remove_w_long();
                        self.z_hash ^= z_castling(self.castlings);
                    }
                    (Color::White, Role::Rook, Square::H1) => {
                        self.z_hash ^= z_castling(self.castlings);
                        self.castlings.remove_w_short();
                        self.z_hash ^= z_castling(self.castlings);
                    }
                    (Color::Black, Role::King, Square::E8) => {
                        self.z_hash ^= z_castling(self.castlings);
                        self.castlings.remove_black();
                        self.z_hash ^= z_castling(self.castlings);
                    }
                    (Color::Black, Role::Rook, Square::A8) => {
                        self.z_hash ^= z_castling(self.castlings);
                        self.castlings.remove_b_long();
                        self.z_hash ^= z_castling(self.castlings);
                    }
                    (Color::Black, Role::Rook, Square::H8) => {
                        self.z_hash ^= z_castling(self.castlings);
                        self.castlings.remove_b_short();
                        self.z_hash ^= z_castling(self.castlings);
                    }

                    _rest => (),
                }

                // zeroify half moves on irreversible move, increment otherwise
                if moving_piece.role() == Role::Pawn || rest.is_capture() {
                    self.half_moves = 0;
                } else {
                    self.half_moves += 1;
                }
            }
        }

        self.z_hash ^= z_black_to_move();

        // increment full_moves
        if us == Color::Black {
            self.full_moves = self.full_moves.saturating_add(1);
        }
        // change the playing side
        self.turn = !self.turn;

        undo
    }

    pub fn reset(&mut self) {
        mem::take(self);
    }

    pub fn to_fen(&self) -> Fen {
        Fen {
            board: self.board.clone(),
            turn: self.turn,
            castlings: self.castlings,
            ep_square: self.ep,
            half_moves: self.half_moves,
            full_moves: self.full_moves,
        }
    }

    /// utility method used for debugging purposes
    ///
    /// lift up some restrictions for FEN
    pub fn health_check(&self) -> Result<(), PositionError> {
        let our = self.turn;
        let enemy = !our;
        let board = &self.board;

        if board.king(our).popcnt() != 1 || board.king(enemy).popcnt() != 1 {
            return Err(PositionError::TooManyKings);
        }

        if board.whites().popcnt() > 16 || board.blacks().popcnt() > 16 {
            return Err(PositionError::TooMuchMaterial);
        }

        for pawn in board.pawns(Color::White) {
            if pawn.rank() == Rank::First {
                return Err(PositionError::InvalidPawn);
            }
        }

        for pawn in board.pawns(Color::Black) {
            if pawn.rank() == Rank::Eighth {
                return Err(PositionError::InvalidPawn);
            }
        }

        if let Some(ep_sqr) = self.ep {
            let (offset, expected_pawn) = match our {
                Color::White => (-8, Piece::BPawn),
                Color::Black => (8, Piece::WPawn),
            };

            let captured_pawn = ep_sqr.offset_checked(offset);

            if board.peek(captured_pawn) != Some(expected_pawn) {
                return Err(PositionError::InvalidEnPassaunt);
            }
        }

        let w_king = board.peek(Square::E1);
        let b_king = board.peek(Square::E8);

        let rights = &self.castlings;

        if rights.w_short()
            && (w_king != Some(Piece::WKing) || board.peek(Square::H1) != Some(Piece::WRook))
        {
            return Err(PositionError::InvalidCastling);
        }

        if rights.w_long()
            && (w_king != Some(Piece::WKing) || board.peek(Square::A1) != Some(Piece::WRook))
        {
            return Err(PositionError::InvalidCastling);
        }

        if rights.b_short()
            && (b_king != Some(Piece::BKing) || board.peek(Square::H8) != Some(Piece::BRook))
        {
            return Err(PositionError::InvalidCastling);
        }

        if rights.b_long()
            && (b_king != Some(Piece::BKing) || board.peek(Square::A8) != Some(Piece::BRook))
        {
            return Err(PositionError::InvalidCastling);
        }

        if self.is_checkmate() || self.is_stalemate() {
            return Err(PositionError::NoLegalMoves);
        }

        Ok(())
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChessBoard")
            .field("board", &self.board)
            .field("turn", &self.turn)
            .field("castlings", &self.castlings)
            .field("ep_square", &self.ep)
            .field("half_moves", &self.half_moves)
            .field("full_moves", &self.full_moves)
            .field("mg_score", &self.mg_score)
            .field("eg_score", &self.eg_score)
            .field("phase", &self.phase)
            .field("z_hash", &self.z_hash)
            .finish()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        let Position {
            board,
            turn,
            castlings,
            ep,
            half_moves,
            full_moves,
            z_hash,
            phase,
            mg_score,
            eg_score,
        } = &*self;

        let Position {
            board: other_board,
            turn: other_turn,
            castlings: other_castlings,
            ep: other_ep,
            half_moves: other_half_moves,
            full_moves: other_full_moves,
            z_hash: other_z_hash,
            phase: other_phase,
            mg_score: other_mg_score,
            eg_score: other_eg_score,
        } = other;

        board == other_board
            && turn == other_turn
            && castlings == other_castlings
            && ep == other_ep
            && half_moves == other_half_moves
            && full_moves == other_full_moves
            && z_hash == other_z_hash
            && phase == other_phase
            && mg_score == other_mg_score
            && eg_score == other_eg_score
    }
}

impl Eq for Position {}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

use super::*;
    use rand::seq::IndexedRandom;

    #[test]
    fn score_test() {
        let fen = Fen::from_str("rn2kbnr/ppp2ppp/6P1/4p3/7P/2N5/PPPP1P2/R1B1K2b w Qkq - 0 12").unwrap();
        let mut pos = fen.try_to_position().unwrap();

        dbg!(&pos);

    }

    #[test]
    fn real_game_test() {
        let mut board = Position::new();

        let moves = vec![
            Move::double_push(Square::D2, Square::D4),
            Move::double_push(Square::D7, Square::D5),
            Move::quiet(Square::G1, Square::F3),
            Move::quiet(Square::B8, Square::C6),
            Move::quiet(Square::B1, Square::C3),
            Move::quiet(Square::G8, Square::F6),
            Move::quiet(Square::E2, Square::E3),
            Move::quiet(Square::E7, Square::E6),
            Move::quiet(Square::A2, Square::A3),
            Move::quiet(Square::G7, Square::G6),
            Move::quiet(Square::F1, Square::B5),
            Move::quiet(Square::A7, Square::A6),
            Move::capture(Square::B5, Square::C6),
            Move::capture(Square::B7, Square::C6),
            Move::quiet(Square::F3, Square::E5),
            Move::quiet(Square::D8, Square::D6),
            Move::quiet(Square::F2, Square::F3),
            Move::quiet(Square::C6, Square::C5),
            Move::king_castle(Square::E1, Square::G1),
            Move::quiet(Square::C5, Square::C4),
            Move::quiet(Square::E3, Square::E4),
            Move::quiet(Square::C7, Square::C6),
            Move::quiet(Square::C1, Square::F4),
            Move::quiet(Square::A6, Square::A5),
            Move::quiet(Square::C3, Square::A4),
            Move::quiet(Square::F6, Square::H5),
            Move::quiet(Square::D1, Square::D2),
            Move::double_push(Square::F7, Square::F5),
            Move::capture(Square::E4, Square::F5),
            Move::capture(Square::E6, Square::F5),
            Move::quiet(Square::F1, Square::E1),
            Move::quiet(Square::C8, Square::D7),
            Move::capture(Square::E5, Square::C6),
            Move::quiet(Square::E8, Square::F7),
            Move::capture(Square::F4, Square::D6),
            Move::capture(Square::F8, Square::D6),
            Move::quiet(Square::C6, Square::E5),
            Move::quiet(Square::F7, Square::G7),
            Move::capture(Square::E5, Square::D7),
            Move::quiet(Square::H7, Square::H6),
            Move::quiet(Square::A4, Square::B6),
            Move::quiet(Square::A8, Square::A7),
            Move::quiet(Square::D7, Square::C5),
            Move::quiet(Square::D6, Square::F4),
            Move::quiet(Square::C5, Square::E6),
            Move::quiet(Square::G7, Square::F6),
            Move::capture(Square::E6, Square::F4),
            Move::capture(Square::H5, Square::F4),
            Move::capture(Square::D2, Square::F4),
            Move::quiet(Square::G6, Square::G5),
            Move::quiet(Square::F4, Square::E5),
            Move::quiet(Square::F6, Square::F7),
            Move::capture(Square::E5, Square::H8),
            Move::quiet(Square::F7, Square::G6),
            Move::double_push(Square::G2, Square::G4),
            Move::quiet(Square::A7, Square::H7),
            Move::capture(Square::H8, Square::H7),
            Move::capture(Square::G6, Square::H7),
            Move::quiet(Square::E1, Square::E7),
            Move::quiet(Square::H7, Square::G6),
            Move::quiet(Square::A1, Square::E1),
            Move::quiet(Square::G6, Square::F6),
            Move::quiet(Square::E1, Square::E6),
        ];


        for mv in moves.into_iter() {
            let _undo = board.do_move(mv).unwrap();
        }
    }

    #[test]
    fn real_game_uci_moves_test() {
        let mut pos = Position::new();

        pos.uci_move_checked("d2d4");
        pos.uci_move_checked("d7d5");
        pos.uci_move_checked("g1f3");
        pos.uci_move_checked("b8c6");
        pos.uci_move_checked("b1c3");
        pos.uci_move_checked("g8f6");
        pos.uci_move_checked("e2e3");
        pos.uci_move_checked("e7e6");
        pos.uci_move_checked("a2a3");
        pos.uci_move_checked("g7g6");
        pos.uci_move_checked("f1b5");
        pos.uci_move_checked("a7a6");
        pos.uci_move_checked("b5c6");
        pos.uci_move_checked("b7c6");
        pos.uci_move_checked("f3e5");
        pos.uci_move_checked("d8d6");
        pos.uci_move_checked("f2f3");
        pos.uci_move_checked("c6c5");
        pos.uci_move_checked("e1g1");
        pos.uci_move_checked("c5c4");
        pos.uci_move_checked("e3e4");
        pos.uci_move_checked("c7c6");
        pos.uci_move_checked("c1f4");
        pos.uci_move_checked("a6a5");
        pos.uci_move_checked("c3a4");
        pos.uci_move_checked("f6h5");
        pos.uci_move_checked("d1d2");
        pos.uci_move_checked("f7f5");
        pos.uci_move_checked("e4f5");
        pos.uci_move_checked("e6f5");
        pos.uci_move_checked("f1e1");
        pos.uci_move_checked("c8d7");
        pos.uci_move_checked("e5c6");
        pos.uci_move_checked("e8f7");
        pos.uci_move_checked("f4d6");
        pos.uci_move_checked("f8d6");
        pos.uci_move_checked("c6e5");
        pos.uci_move_checked("f7g7");
        pos.uci_move_checked("e5d7");
        pos.uci_move_checked("h7h6");
        pos.uci_move_checked("a4b6");
        pos.uci_move_checked("a8a7");
        pos.uci_move_checked("d7c5");
        pos.uci_move_checked("d6f4");
        pos.uci_move_checked("c5e6");
        pos.uci_move_checked("g7f6");
        pos.uci_move_checked("e6f4");
        pos.uci_move_checked("h5f4");
        pos.uci_move_checked("d2f4");
        pos.uci_move_checked("g6g5");
        pos.uci_move_checked("f4e5");
        pos.uci_move_checked("f6f7");
        pos.uci_move_checked("e5h8");
        pos.uci_move_checked("f7g6");
        pos.uci_move_checked("g2g4");
        pos.uci_move_checked("a7h7");
        pos.uci_move_checked("h8h7");
        pos.uci_move_checked("g6h7");
        pos.uci_move_checked("e1e7");
        pos.uci_move_checked("h7g6");
        pos.uci_move_checked("a1e1");
        pos.uci_move_checked("g6f6");
        pos.uci_move_checked("e1e6");
    }

    #[test]
    fn zobrist_test() {
        let mut board = Position::new();

        let moves = vec![
            Move::double_push(Square::D2, Square::D4),
            Move::double_push(Square::D7, Square::D5),
            Move::quiet(Square::G1, Square::F3),
            Move::quiet(Square::B8, Square::C6),
            Move::quiet(Square::B1, Square::C3),
            Move::quiet(Square::G8, Square::F6),
            Move::quiet(Square::E2, Square::E3),
            Move::quiet(Square::E7, Square::E6),
            Move::quiet(Square::A2, Square::A3),
            Move::quiet(Square::G7, Square::G6),
            Move::quiet(Square::F1, Square::B5),
            Move::quiet(Square::A7, Square::A6),
            Move::capture(Square::B5, Square::C6),
            Move::capture(Square::B7, Square::C6),
            Move::quiet(Square::F3, Square::E5),
            Move::quiet(Square::D8, Square::D6),
            Move::quiet(Square::F2, Square::F3),
            Move::quiet(Square::C6, Square::C5),
            Move::king_castle(Square::E1, Square::G1),
            Move::quiet(Square::C5, Square::C4),
            Move::quiet(Square::E3, Square::E4),
            Move::quiet(Square::C7, Square::C6),
            Move::quiet(Square::C1, Square::F4),
            Move::quiet(Square::A6, Square::A5),
            Move::quiet(Square::C3, Square::A4),
            Move::quiet(Square::F6, Square::H5),
            Move::quiet(Square::D1, Square::D2),
            Move::double_push(Square::F7, Square::F5),
            Move::capture(Square::E4, Square::F5),
            Move::capture(Square::E6, Square::F5),
            Move::quiet(Square::F1, Square::E1),
            Move::quiet(Square::C8, Square::D7),
            Move::capture(Square::E5, Square::C6),
            Move::quiet(Square::E8, Square::F7),
            Move::capture(Square::F4, Square::D6),
            Move::capture(Square::F8, Square::D6),
            Move::quiet(Square::C6, Square::E5),
            Move::quiet(Square::F7, Square::G7),
            Move::capture(Square::E5, Square::D7),
            Move::quiet(Square::H7, Square::H6),
            Move::quiet(Square::A4, Square::B6),
            Move::quiet(Square::A8, Square::A7),
            Move::quiet(Square::D7, Square::C5),
            Move::quiet(Square::D6, Square::F4),
            Move::quiet(Square::C5, Square::E6),
            Move::quiet(Square::G7, Square::F6),
            Move::capture(Square::E6, Square::F4),
            Move::capture(Square::H5, Square::F4),
            Move::capture(Square::D2, Square::F4),
            Move::quiet(Square::G6, Square::G5),
            Move::quiet(Square::F4, Square::E5),
            Move::quiet(Square::F6, Square::F7),
            Move::capture(Square::E5, Square::H8),
            Move::quiet(Square::F7, Square::G6),
            Move::double_push(Square::G2, Square::G4),
            Move::quiet(Square::A7, Square::H7),
            Move::capture(Square::H8, Square::H7),
            Move::capture(Square::G6, Square::H7),
            Move::quiet(Square::E1, Square::E7),
            Move::quiet(Square::H7, Square::G6),
            Move::quiet(Square::A1, Square::E1),
            Move::quiet(Square::G6, Square::F6),
            Move::quiet(Square::E1, Square::E6),
        ];

        for mv in moves {
            board.do_move(mv).unwrap();
            let updated_hash = board.zobrist_hash();
            let computed_hash = zobrist::compute_hash(&board);

            assert_eq!(updated_hash, computed_hash, "prev move: {mv:?}");
        }
    }

    #[test]
    fn undo_basic_move_restores_position() {
        let mut board = Position::new();

        let initial_hash = board.zobrist_hash();
        let initial_fen = board.to_fen();

        let mv = Move::double_push(Square::E2, Square::E4);

        let undo = board.do_move(mv).unwrap();

        assert_ne!(board.zobrist_hash(), initial_hash);

        unsafe { board.undo_move(undo) };

        let board_after_undo = board.clone();
        assert_eq!(board.zobrist_hash(), initial_hash);
        assert_eq!(board.to_fen(), initial_fen);
        assert_eq!(board_after_undo, board);
    }

    #[test]
    fn undo_capture_restores_piece() {
        let mut board = Position::new();

        board
            .do_move(Move::double_push(Square::E2, Square::E4))
            .unwrap();
        board
            .do_move(Move::double_push(Square::D7, Square::D5))
            .unwrap();

        let mv = Move::capture(Square::E4, Square::D5);

        let before = board.clone();
        let hash_before = board.zobrist_hash();

        let undo = board.do_move(mv).unwrap();
        unsafe { board.undo_move(undo) };

        assert_eq!(board, before);
        assert_eq!(board.zobrist_hash(), hash_before);
    }

    #[test]
    fn undo_promotion_restores_pawn() {
        let mut board = Position::new();

        board.uci_move_checked("b2b4");
        board.uci_move_checked("a7a6");
        board.uci_move_checked("b4b5");
        board.uci_move_checked("h7h6");
        board.uci_move_checked("b5a6");
        board.uci_move_checked("h6h5");
        board.uci_move_checked("a6b7");
        board.uci_move_checked("h5h4");

        let before = board.clone();

        let undo = board
            .do_move(Move::promotion(Square::B7, Square::A8, Role::Queen, true))
            .unwrap();

        unsafe { board.undo_move(undo) };

        assert_eq!(board, before);
    }

    #[test]
    fn undo_castling_restores_king_and_rook() {
        let mut board = Position::new();

        board.uci_move_checked("e2e3");
        board.uci_move_checked("e7e6");
        board.uci_move_checked("f1d3");
        board.uci_move_checked("a7a6");
        board.uci_move_checked("g1f3");
        board.uci_move_checked("b7b6");

        let before = board.clone();

        let undo = board
            .do_move(Move::king_castle(Square::E1, Square::G1))
            .unwrap();

        unsafe { board.undo_move(undo) };

        assert_eq!(board, before);
    }

    #[test]
    fn undo_en_passant_restores_captured_pawn() {
        let mut board = Position::new();

        board.uci_move_checked("e2e4");
        board.uci_move_checked("a7a6");
        board.uci_move_checked("e4e5");
        board.uci_move_checked("d7d5");

        let before = board.clone();

        let undo = board
            .do_move(Move::en_passant(Square::E5, Square::D6))
            .unwrap();

        unsafe { board.undo_move(undo) };

        assert_eq!(board, before);
    }

    #[test]
    fn undo_restores_turn_correctly() {
        let mut board = Position::new();

        let before = board.clone();
        let start_turn = board.turn();

        let undo = board
            .do_move(Move::double_push(Square::E2, Square::E4))
            .unwrap();

        assert_ne!(board.turn(), start_turn);

        unsafe { board.undo_move(undo) };
        assert_eq!(board.turn(), start_turn);
        assert_eq!(board, before);
    }

    #[test]
    fn random_move_undo_roundtrip() {
        let mut board = Position::new();
        let before = board.clone();

        for _ in 0..100_000 {
            let moves = board.legal_moves();
            let mv = moves.choose(&mut rand::rng()).unwrap();

            let prev_hash = board.zobrist_hash();

            let undo = board.do_move(*mv).unwrap();
            unsafe { board.undo_move(undo) };

            assert_eq!(board.zobrist_hash(), prev_hash);
        }

        assert_eq!(board, before)
    }
}
