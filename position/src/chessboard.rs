use std::{f32::MANTISSA_DIGITS, fmt::Debug, num::NonZeroU32};

use arrayvec::ArrayVec;
use types::{
    bitboard::Bitboard, castling_rights::CastlingRights, chess_move::Move, color::Color,
    file::File, piece::Piece, rank::Rank, role::Role, square::Square,
};

use crate::{board::Board, fen::Fen, move_gen};

// TODO make richer, return move that is invalid, unchanged chessboard
#[derive(Debug)]
pub struct InvalidMoveError;

#[derive(Debug)]
pub enum PositionError {
    TooManyKings,
    TooMuchMaterial,
    InvalidCastling,
    InvalidEnPassaunt,
    NoLegalMoves,
    InvalidPawn,
}

#[derive(Clone)]
pub struct ChessBoard {
    board: Board,
    turn: Color,
    castling_rights: CastlingRights,
    ep_square: Option<Square>,
    half_moves: u32,
    full_moves: NonZeroU32,
}

impl ChessBoard {
    /// create a new chessboard with standart position
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            turn: Color::White,
            castling_rights: CastlingRights::new(),
            ep_square: None,
            half_moves: 0,
            full_moves: NonZeroU32::MIN,
        }
    }

    pub fn from_fen(
        Fen {
            board,
            turn,
            castling_rights,
            ep_square,
            half_moves,
            full_moves,
            ..
        }: Fen,
    ) -> Result<Self, PositionError> {
        let pos = Self {
            board,
            castling_rights,
            turn,
            ep_square,
            half_moves,
            full_moves,
        };

        pos.health_check()?;

        Ok(pos)
    }

    /// Returns a bitboard of pieces giving check to the king
    #[inline(always)]
    pub fn checkers(&self, side: Color) -> Bitboard {
        let king = self.board.the_king(side);
        self.board.attacks_to(king, !side)
    }

    pub fn is_checkmate(&self) -> bool {
        self.checkers(self.turn).present() && self.legal_moves().is_empty()
    }

    pub fn is_stalemate(&self) -> bool {
        self.checkers(self.turn).empty() && self.legal_moves().is_empty()
    }

    pub fn half_moves(&self) -> u32 {
        self.half_moves
    }

    pub fn full_moves(&self) -> u32 {
        self.full_moves.get()
    }

    /// Generate and return a list of legal moves for the curent position
    pub fn legal_moves(&self) -> ArrayVec<Move, 218> {
        move_gen::gen_legal_moves(self)
    }

    #[inline(always)]
    pub const fn turn(&self) -> Color {
        self.turn
    }

    #[inline(always)]
    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn ep_square(&self) -> Option<Square> {
        self.ep_square
    }

    pub fn castling_rights(&self) -> &CastlingRights {
        &self.castling_rights
    }

    /// Checks move for legality and then executes it
    ///
    /// consider do_move_inner_checked if you can guarantee validity
    pub fn do_move(&mut self, mv: Move) -> Result<(), InvalidMoveError> {
        if self.is_legal_move(mv) {
            self.do_move_inner_checked(mv);
            Ok(())
        } else {
            Err(InvalidMoveError)
        }
    }

    pub fn uci_move(&mut self, raw_uci: &str) -> Result<(), InvalidMoveError> {
        let uci_move = self.parse_uci(raw_uci).ok_or(InvalidMoveError)?;

        self.do_move_inner_checked(uci_move);

        Ok(())
    }

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

    /// # PANICS
    /// Calling this method without validity quarantees by the caller may corrupt the state of ChessBoard
    ///
    /// which may lead to following:
    ///
    /// **ANY** subsequent call of **ANY** method of ChessBoard can panic at **ANY** time
    #[rustfmt::skip]
    pub fn do_move_inner_checked(&mut self, mv: Move) {
        self.ep_square.take();

        let us = self.turn;
        let board = &mut self.board;

        match mv {
            Move::Standart { role, from, to, capture, promotion, } => {
                // set en_passaunt
                if role == Role::Pawn && (to as i32 - from as i32).abs() == 16 {
                    let ep = ((from as u8 + to as u8) >> 1) as u32;
                    self.ep_square = Some(Square::from_u32_checked(ep));
                }

                board.discard_piece_at(from);

                if capture.is_some() {
                    board.discard_piece_at(to);
                }

                // if prom exists, set it at 'to' sqr, set moving piece otherwise

                let piece = match promotion {
                    Some(promo) => Piece::of(promo, us),
                    None => Piece::of(role, us),
                };

                board.set_piece_at(piece, to);

                // change castling rights
                match (capture, to.file(), to.rank()) {
                    (Some(Role::Rook), File::H, Rank::First) if us == Color::Black => {
                        self.castling_rights.remove_w_short()
                    }
                    (Some(Role::Rook), File::H, Rank::Eighth) if us == Color::White => {
                        self.castling_rights.remove_b_short();
                    }
                    (Some(Role::Rook), File::A, Rank::First) if us == Color::Black => {
                        self.castling_rights.remove_w_long();
                    }
                    (Some(Role::Rook), File::A, Rank::Eighth) if us == Color::White => {
                        self.castling_rights.remove_b_long();
                    }
                    _ => (),
                }

                // change castling rights
                match (us, role, from) {
                    (Color::White, Role::King, Square::E1) => self.castling_rights.remove_white(),
                    (Color::White, Role::Rook, Square::A1) => self.castling_rights.remove_w_long(),
                    (Color::White, Role::Rook, Square::H1) => self.castling_rights.remove_w_short(),
                    (Color::Black, Role::King, Square::E8) => self.castling_rights.remove_black(),
                    (Color::Black, Role::Rook, Square::A8) => self.castling_rights.remove_b_long(),
                    (Color::Black, Role::Rook, Square::H8) => self.castling_rights.remove_b_short(),

                    _rest => (),
                }

                if role == Role::Pawn || capture.is_some() {
                    self.half_moves = 0;
                } else {
                    self.half_moves += 1;
                }
            }
            Move::EnPassant { from, to } => {
                self.half_moves = 0;
                let our_pawn = Piece::of(Role::Pawn, us);
                let captured_pawn = Square::of(to.file(), from.rank());

                board.discard_piece_at(captured_pawn);
                board.discard_piece_at(from);
                board.set_piece_at(our_pawn, to);
            }
            Move::Castling { king, rook } => {
                let (king_dest, rook_dest) = match rook {
                    Square::H1 => (Square::G1, Square::F1),
                    Square::A1 => (Square::C1, Square::D1),
                    Square::H8 => (Square::G8, Square::F8),
                    Square::A8 => (Square::C8, Square::D8),
                    _illegal => unreachable!("Illegal rook square for castling"),
                };

                board.discard_piece_at(king);
                board.discard_piece_at(rook);

                board.set_piece_at(Piece::of(Role::King, us), king_dest);
                board.set_piece_at(Piece::of(Role::Rook, us), rook_dest);

                self.castling_rights.remove_all_of(us);

                self.half_moves += 1;
            }
        }

        // increment full_moves
        if us == Color::Black {
            self.full_moves = self.full_moves.saturating_add(1);
        }

        // change the playing side
        self.turn = !self.turn;
    }

    pub fn into_fen(&self) -> Fen {
        Fen {
            board: self.board.clone(),
            turn: self.turn,
            castling_rights: self.castling_rights.clone(),
            ep_square: self.ep_square,
            half_moves: self.half_moves,
            full_moves: self.full_moves,
        }
    }

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

        if let Some(ep_sqr) = self.ep_square {
            let (offset, expected_pawn) = match our {
                Color::White => (-8, Piece::WPawn),
                Color::Black => (8, Piece::BPawn),
            };

            let captured_pawn = ep_sqr.offset_checked(offset);

            if board.peek(captured_pawn) != Some(expected_pawn) {
                return Err(PositionError::InvalidEnPassaunt);
            }
        }

        let w_king = board.peek(Square::E1);
        let b_king = board.peek(Square::E8);

        let rights = &self.castling_rights;

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

impl Debug for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChessBoard")
            .field("board", &self.board)
            .field("turn", &self.turn)
            .field("castling_rights", &self.castling_rights)
            .field("ep_square", &self.ep_square)
            .field("half_moves", &self.half_moves)
            .field("full_moves", &self.full_moves)
            .finish()
    }
}

impl Default for ChessBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use shakmaty::fen::BoardFen;
    use types::chess_move::CastlingSide;

    use super::*;

    #[test]
    fn real_game_test() {
        let mut board = ChessBoard::new();
        println!("{board:#?}");

        let moves = vec![
            Move::quiet(Role::Pawn, Square::D2, Square::D4),
            Move::quiet(Role::Pawn, Square::D7, Square::D5),
            Move::quiet(Role::Knight, Square::G1, Square::F3),
            Move::quiet(Role::Knight, Square::B8, Square::C6),
            Move::quiet(Role::Knight, Square::B1, Square::C3),
            Move::quiet(Role::Knight, Square::G8, Square::F6),
            Move::quiet(Role::Pawn, Square::E2, Square::E3),
            Move::quiet(Role::Pawn, Square::E7, Square::E6),
            Move::quiet(Role::Pawn, Square::A2, Square::A3),
            Move::quiet(Role::Pawn, Square::G7, Square::G6),
            Move::quiet(Role::Bishop, Square::F1, Square::B5),
            Move::quiet(Role::Pawn, Square::A7, Square::A6),
            Move::capture(Role::Bishop, Square::B5, Square::C6, Role::Knight),
            Move::capture(Role::Pawn, Square::B7, Square::C6, Role::Bishop),
            Move::quiet(Role::Knight, Square::F3, Square::E5),
            Move::quiet(Role::Queen, Square::D8, Square::D6),
            Move::quiet(Role::Pawn, Square::F2, Square::F3),
            Move::quiet(Role::Pawn, Square::C6, Square::C5),
            Move::castling(CastlingSide::WShort),
            Move::quiet(Role::Pawn, Square::C5, Square::C4),
            Move::quiet(Role::Pawn, Square::E3, Square::E4),
            Move::quiet(Role::Pawn, Square::C7, Square::C6),
            Move::quiet(Role::Bishop, Square::C1, Square::F4),
            Move::quiet(Role::Pawn, Square::A6, Square::A5),
            Move::quiet(Role::Knight, Square::C3, Square::A4),
            Move::quiet(Role::Knight, Square::F6, Square::H5),
            Move::quiet(Role::Queen, Square::D1, Square::D2),
            Move::quiet(Role::Pawn, Square::F7, Square::F5),
            Move::capture(Role::Pawn, Square::E4, Square::F5, Role::Pawn),
            Move::capture(Role::Pawn, Square::E6, Square::F5, Role::Pawn),
            Move::quiet(Role::Rook, Square::F1, Square::E1),
            Move::quiet(Role::Bishop, Square::C8, Square::D7),
            Move::capture(Role::Knight, Square::E5, Square::C6, Role::Pawn),
            Move::quiet(Role::King, Square::E8, Square::F7),
            Move::capture(Role::Bishop, Square::F4, Square::D6, Role::Queen),
            Move::capture(Role::Bishop, Square::F8, Square::D6, Role::Bishop),
            Move::quiet(Role::Knight, Square::C6, Square::E5),
            Move::quiet(Role::King, Square::F7, Square::G7),
            Move::capture(Role::Knight, Square::E5, Square::D7, Role::Bishop),
            Move::quiet(Role::Pawn, Square::H7, Square::H6),
            Move::quiet(Role::Knight, Square::A4, Square::B6),
            Move::quiet(Role::Rook, Square::A8, Square::A7),
            Move::quiet(Role::Knight, Square::D7, Square::C5),
            Move::quiet(Role::Bishop, Square::D6, Square::F4),
            Move::quiet(Role::Knight, Square::C5, Square::E6),
            Move::quiet(Role::King, Square::G7, Square::F6),
            Move::capture(Role::Knight, Square::E6, Square::F4, Role::Bishop),
            Move::capture(Role::Knight, Square::H5, Square::F4, Role::Knight),
            Move::capture(Role::Queen, Square::D2, Square::F4, Role::Knight),
            Move::quiet(Role::Pawn, Square::G6, Square::G5),
            Move::quiet(Role::Queen, Square::F4, Square::E5),
            Move::quiet(Role::King, Square::F6, Square::F7),
            Move::capture(Role::Queen, Square::E5, Square::H8, Role::Rook),
            Move::quiet(Role::King, Square::F7, Square::G6),
            Move::quiet(Role::Pawn, Square::G2, Square::G4),
            Move::quiet(Role::Rook, Square::A7, Square::H7),
            Move::capture(Role::Queen, Square::H8, Square::H7, Role::Rook),
            Move::capture(Role::King, Square::G6, Square::H7, Role::Queen),
            Move::quiet(Role::Rook, Square::E1, Square::E7),
            Move::quiet(Role::King, Square::H7, Square::G6),
            Move::quiet(Role::Rook, Square::A1, Square::E1),
            Move::quiet(Role::King, Square::G6, Square::F6),
            Move::quiet(Role::Rook, Square::E1, Square::E6),
        ];

        for (idx, mv) in moves.into_iter().enumerate() {
            board.do_move(mv).unwrap();
        }
    }

    #[test]
    fn real_game_uci_moves_test() {
        let mut board = ChessBoard::new();
        println!("{board:#?}");

        board.uci_move("d2d4").unwrap();
        board.uci_move("d7d5").unwrap();
        board.uci_move("g1f3").unwrap();
        board.uci_move("b8c6").unwrap();
        board.uci_move("b1c3").unwrap();
        board.uci_move("g8f6").unwrap();
        board.uci_move("e2e3").unwrap();
        board.uci_move("e7e6").unwrap();
        board.uci_move("a2a3").unwrap();
        board.uci_move("g7g6").unwrap();
        board.uci_move("f1b5").unwrap();
        board.uci_move("a7a6").unwrap();
        board.uci_move("b5c6").unwrap();
        board.uci_move("b7c6").unwrap();
        board.uci_move("f3e5").unwrap();
        board.uci_move("d8d6").unwrap();
        board.uci_move("f2f3").unwrap();
        board.uci_move("c6c5").unwrap();
        board.uci_move("e1g1").unwrap();
        board.uci_move("c5c4").unwrap();
        board.uci_move("e3e4").unwrap();
        board.uci_move("c7c6").unwrap();
        board.uci_move("c1f4").unwrap();
        board.uci_move("a6a5").unwrap();
        board.uci_move("c3a4").unwrap();
        board.uci_move("f6h5").unwrap();
        board.uci_move("d1d2").unwrap();
        board.uci_move("f7f5").unwrap();
        board.uci_move("e4f5").unwrap();
        board.uci_move("e6f5").unwrap();
        board.uci_move("f1e1").unwrap();
        board.uci_move("c8d7").unwrap();
        board.uci_move("e5c6").unwrap();
        board.uci_move("e8f7").unwrap();
        board.uci_move("f4d6").unwrap();
        board.uci_move("f8d6").unwrap();
        board.uci_move("c6e5").unwrap();
        board.uci_move("f7g7").unwrap();
        board.uci_move("e5d7").unwrap();
        board.uci_move("h7h6").unwrap();
        board.uci_move("a4b6").unwrap();
        board.uci_move("a8a7").unwrap();
        board.uci_move("d7c5").unwrap();
        board.uci_move("d6f4").unwrap();
        board.uci_move("c5e6").unwrap();
        board.uci_move("g7f6").unwrap();
        board.uci_move("e6f4").unwrap();
        board.uci_move("h5f4").unwrap();
        board.uci_move("d2f4").unwrap();
        board.uci_move("g6g5").unwrap();
        board.uci_move("f4e5").unwrap();
        board.uci_move("f6f7").unwrap();
        board.uci_move("e5h8").unwrap();
        board.uci_move("f7g6").unwrap();
        board.uci_move("g2g4").unwrap();
        board.uci_move("a7h7").unwrap();
        board.uci_move("h8h7").unwrap();
        board.uci_move("g6h7").unwrap();
        board.uci_move("e1e7").unwrap();
        board.uci_move("h7g6").unwrap();
        board.uci_move("a1e1").unwrap();
        board.uci_move("g6f6").unwrap();
        board.uci_move("e1e6").unwrap();
    }

    #[test]
    fn move_generation_tests() {
        let board = ChessBoard::new();
        let moves = board.legal_moves();
        println!("{}", moves.len());

        for mv in moves {
            println!("{}", mv.to_uci());
        }
    }
}
