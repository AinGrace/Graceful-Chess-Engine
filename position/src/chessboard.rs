use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    mem,
    num::NonZeroU32,
};

use types::{
    MoveList, bitboard::Bitboard, castlings::Castlings, chess_move::Move, color::Color,
    piece::Piece, rank::Rank, role::Role, square::Square,
};

use crate::{board::Board, fen::Fen, move_gen, zobrist};

pub enum GameResult {
    White,
    Black,
    Draw,
    Unknown,
}

impl GameResult {
    fn new_winner(side: Color) -> Self {
        match side {
            Color::White => Self::White,
            Color::Black => Self::Black,
        }
    }
}

#[derive(Debug)]
pub struct InvalidMoveError {
    mv: String,
    board: ChessBoard,
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

    pub fn chessboard_ref(&self) -> &ChessBoard {
        &self.board
    }

    pub fn chessboard(self) -> ChessBoard {
        self.board
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

#[derive(Clone)]
pub struct Undo {
    m: Move,
    castlings: Castlings,
    ep_square: Option<Square>,
    half_moves: u32,
    full_moves: NonZeroU32,
    zobrist_hash: u64,
}

/// introduce undo
#[derive(Clone)]
pub struct ChessBoard {
    board: Board,
    turn: Color,
    castlings: Castlings,
    ep_square: Option<Square>,
    half_moves: u32,
    full_moves: NonZeroU32,
    zobrist_hash: u64,
    zobrist_hashes: HashMap<u64, u8>,

    history: Vec<Undo>,
}

impl ChessBoard {
    /// create a new chessboard with standart position
    pub fn new() -> Self {
        let mut pos = Self {
            board: Board::new(),
            turn: Color::White,
            castlings: Castlings::new(),
            ep_square: None,
            half_moves: 0,
            full_moves: NonZeroU32::MIN,
            zobrist_hash: 0, // temporary value
            zobrist_hashes: HashMap::new(),

            history: vec![],
        };

        let z_hash = zobrist::compute_hash(&pos);
        pos.zobrist_hash = z_hash;
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
            ep_square,
            half_moves,
            full_moves,
            zobrist_hash: 0, //temporary value
            zobrist_hashes: HashMap::new(),

            history: vec![],
        };

        pos.health_check()?;

        let z_hash = zobrist::compute_hash(&pos);
        pos.zobrist_hash = z_hash;

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

    pub fn zobrist_hash(&self) -> u64 {
        self.zobrist_hash
    }

    /// Generate and return a list of legal moves for the curent position
    pub fn legal_moves(&self) -> MoveList {
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

    pub fn castling_rights(&self) -> &Castlings {
        &self.castlings
    }

    /// Checks move for legality and then executes it
    ///
    /// consider do_move_inner_checked if you can guarantee validity
    pub fn do_move(mut self, mv: Move) -> Result<Self, InvalidMoveError> {
        if self.is_legal_move(mv) {
            let undo = Undo {
                m: mv,
                castlings: self.castlings,
                ep_square: self.ep_square(),
                half_moves: self.half_moves,
                full_moves: self.full_moves,
                zobrist_hash: self.zobrist_hash,
            };

            self.history.push(undo);
            self.do_move_inner(mv);

            Ok(self)
        } else {
            // TODO consider using long algebraic notation instead of uci
            Err(InvalidMoveError {
                mv: mv.to_uci(),
                board: self,
            })
        }
    }

    /// parse uci string and execute it
    pub fn uci_move(mut self, raw_uci: &str) -> Result<Self, InvalidMoveError> {
        let uci_move = match self.parse_uci(raw_uci) {
            Some(uci_move) => uci_move,
            None => {
                return Err(InvalidMoveError {
                    mv: raw_uci.into(),
                    board: self,
                });
            }
        };

        self.do_move_inner(uci_move);

        Ok(self)
    }

    pub fn uci_move_checked(mut self, raw_uci: &str) -> Self {
        let uci_move = self
            .parse_uci(raw_uci)
            .expect("caller quarantees validity of raw_uci");

        self.do_move_inner(uci_move);
        self
    }

    pub fn undo_move(&mut self) {
        let Some(undo) = self.history.pop() else {
            return;
        };

        self.turn = !self.turn;

        let board = &mut self.board;
        match undo.m {
            Move::Standard {
                role: _role,
                from,
                to,
                capture,
                promotion,
            } => {
                let our_piece = board
                    .remove_piece_at(to)
                    .expect("a piece is quaranteed to be there");

                if promotion.is_some() {
                    let our_pawn = if self.turn == Color::White {
                        Piece::WPawn
                    } else {
                        Piece::BPawn
                    };
                    board.set_piece_at(our_pawn, from);
                } else {
                    board.set_piece_at(our_piece, from);
                }

                if let Some(captured) = capture {
                    board.set_piece_at(captured.to_piece(!self.turn), to);
                }
            }
            Move::EnPassant { from, to } => {
                let our_pawn = board.remove_piece_at(to).expect("pawn is quaranteed to be");
                let enemy_pawn = if self.turn == Color::White {
                    Piece::WPawn
                } else {
                    Piece::BPawn
                };

                board.set_piece_at(our_pawn, from);
                board.set_piece_at(enemy_pawn, Square::of(to.file(), from.rank()));
            }
            Move::Castling { king, rook } => {
                let (king_dest, rook_dest) = match rook {
                    Square::H1 => (Square::G1, Square::F1),
                    Square::A1 => (Square::C1, Square::D1),
                    Square::H8 => (Square::G8, Square::F8),
                    Square::A8 => (Square::C8, Square::D8),
                    _illegal => unreachable!("Illegal rook square for castling"),
                };

                let king_piece = board
                    .remove_piece_at(king_dest)
                    .expect("king is quaranteed to be");
                let rook_piece = board
                    .remove_piece_at(rook_dest)
                    .expect("rook is quaranteed to be");

                board.set_piece_at(king_piece, king);
                board.set_piece_at(rook_piece, rook);
            }
        }

        self.castlings = undo.castlings;
        self.ep_square = undo.ep_square;
        self.half_moves = undo.half_moves;
        self.full_moves = undo.full_moves;

        let current_hash = self.zobrist_hash;

        if let Some(counter) = self.zobrist_hashes.get_mut(&current_hash) {
            *counter -= 1;

            if *counter == 0 {
                self.zobrist_hashes.remove(&current_hash);
            }
        }

        self.zobrist_hash = undo.zobrist_hash;
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

    pub fn game_result(&self) -> GameResult {
        if self.board().is_insufficient_material() {
            return GameResult::Draw;
        }

        if self.half_moves >= 100 {
            return GameResult::Draw;
        }

        if self.zobrist_hashes.values().any(|val| *val >= 3) {
            return GameResult::Draw;
        }

        if self.is_checkmate() {
            return GameResult::new_winner(!self.turn);
        }

        if self.is_stalemate() {
            return GameResult::Draw;
        }

        GameResult::Unknown
    }

    /// # PANICS
    /// Calling this method without validity quarantees by the caller may corrupt the state of ChessBoard
    ///
    /// which may lead to following:
    ///
    /// **ANY** subsequent call of **ANY** method of ChessBoard can panic at **ANY** time
    pub fn do_move_inner(&mut self, mv: Move) {
        let us = self.turn;
        let board = &mut self.board;

        match mv {
            Move::Standard {
                role,
                from,
                to,
                capture,
                promotion,
            } => {
                // set en_passaunt on pawn double push
                if role == Role::Pawn && Square::abs_diff(from, to) == 16 {
                    let ep = ((from as u8 + to as u8) >> 1) as u32;
                    self.ep_square = Some(Square::from_u32_checked(ep));
                }

                board.discard_piece_at(from);

                if let Some(captured) = capture {
                    board.discard_piece_at(to);

                    if captured == Role::Rook {
                        // change castling rights on capture
                        match (us, to) {
                            (Color::Black, Square::H1) => self.castlings.remove_w_short(),
                            (Color::White, Square::H8) => self.castlings.remove_b_short(),
                            (Color::Black, Square::A1) => self.castlings.remove_w_long(),
                            (Color::White, Square::A8) => self.castlings.remove_b_long(),

                            _rest => (),
                        }
                    }
                }

                // if promotion exists set it at destination square, otherwise set the moving piece
                let piece = match promotion {
                    Some(promo) => Piece::of(promo, us),
                    None => Piece::of(role, us),
                };

                board.set_piece_at(piece, to);

                // change castling rights on quiet move
                match (us, role, from) {
                    (Color::White, Role::King, Square::E1) => self.castlings.remove_white(),
                    (Color::White, Role::Rook, Square::A1) => self.castlings.remove_w_long(),
                    (Color::White, Role::Rook, Square::H1) => self.castlings.remove_w_short(),
                    (Color::Black, Role::King, Square::E8) => self.castlings.remove_black(),
                    (Color::Black, Role::Rook, Square::A8) => self.castlings.remove_b_long(),
                    (Color::Black, Role::Rook, Square::H8) => self.castlings.remove_b_short(),

                    _rest => (),
                }

                // zeroify half moves on irreversible move, increment otherwise
                if role == Role::Pawn || capture.is_some() {
                    self.half_moves = 0;
                } else {
                    self.half_moves += 1;
                }
            }
            Move::EnPassant { from, to } => {
                self.half_moves = 0;
                let our_pawn = Piece::of(Role::Pawn, us);
                let captured_sqr = Square::of(to.file(), from.rank());

                board.discard_piece_at(captured_sqr);
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

                self.castlings.remove_all_of(us);

                self.half_moves += 1;
            }
        }

        let old_ep = self.ep_square.take(); // remove the ep square
        let old_castling = self.castlings;

        zobrist::update_hash(
            &mut self.zobrist_hash,
            mv,
            us,
            old_ep,
            old_castling,
            self.ep_square,
            self.castlings,
        );

        self.zobrist_hashes
            .entry(self.zobrist_hash)
            .and_modify(|e| *e += 1)
            .or_insert(1);

        // increment full_moves
        if us == Color::Black {
            self.full_moves = self.full_moves.saturating_add(1);
        }

        // change the playing side
        self.turn = !self.turn;
    }

    pub fn reset(&mut self) {
        mem::take(self);
    }

    pub fn into_fen(&self) -> Fen {
        Fen {
            board: self.board.clone(),
            turn: self.turn,
            castlings: self.castlings,
            ep_square: self.ep_square,
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

impl Debug for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChessBoard")
            .field("board", &self.board)
            .field("turn", &self.turn)
            .field("castlings", &self.castlings)
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

        for mv in moves.into_iter() {
            board = board.do_move(mv).unwrap();
        }
    }

    #[test]
    fn real_game_uci_moves_test() {
        let board = ChessBoard::new();

        board
            .uci_move_checked("d2d4")
            .uci_move_checked("d7d5")
            .uci_move_checked("g1f3")
            .uci_move_checked("b8c6")
            .uci_move_checked("b1c3")
            .uci_move_checked("g8f6")
            .uci_move_checked("e2e3")
            .uci_move_checked("e7e6")
            .uci_move_checked("a2a3")
            .uci_move_checked("g7g6")
            .uci_move_checked("f1b5")
            .uci_move_checked("a7a6")
            .uci_move_checked("b5c6")
            .uci_move_checked("b7c6")
            .uci_move_checked("f3e5")
            .uci_move_checked("d8d6")
            .uci_move_checked("f2f3")
            .uci_move_checked("c6c5")
            .uci_move_checked("e1g1")
            .uci_move_checked("c5c4")
            .uci_move_checked("e3e4")
            .uci_move_checked("c7c6")
            .uci_move_checked("c1f4")
            .uci_move_checked("a6a5")
            .uci_move_checked("c3a4")
            .uci_move_checked("f6h5")
            .uci_move_checked("d1d2")
            .uci_move_checked("f7f5")
            .uci_move_checked("e4f5")
            .uci_move_checked("e6f5")
            .uci_move_checked("f1e1")
            .uci_move_checked("c8d7")
            .uci_move_checked("e5c6")
            .uci_move_checked("e8f7")
            .uci_move_checked("f4d6")
            .uci_move_checked("f8d6")
            .uci_move_checked("c6e5")
            .uci_move_checked("f7g7")
            .uci_move_checked("e5d7")
            .uci_move_checked("h7h6")
            .uci_move_checked("a4b6")
            .uci_move_checked("a8a7")
            .uci_move_checked("d7c5")
            .uci_move_checked("d6f4")
            .uci_move_checked("c5e6")
            .uci_move_checked("g7f6")
            .uci_move_checked("e6f4")
            .uci_move_checked("h5f4")
            .uci_move_checked("d2f4")
            .uci_move_checked("g6g5")
            .uci_move_checked("f4e5")
            .uci_move_checked("f6f7")
            .uci_move_checked("e5h8")
            .uci_move_checked("f7g6")
            .uci_move_checked("g2g4")
            .uci_move_checked("a7h7")
            .uci_move_checked("h8h7")
            .uci_move_checked("g6h7")
            .uci_move_checked("e1e7")
            .uci_move_checked("h7g6")
            .uci_move_checked("a1e1")
            .uci_move_checked("g6f6")
            .uci_move_checked("e1e6");
    }

    #[test]
    fn zobrist_test() {
        let mut board = ChessBoard::new();

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

        for mv in moves {
            board.do_move_inner(mv);
            let updated_hash = board.zobrist_hash();
            let computed_hash = zobrist::compute_hash(&board);

            assert_eq!(updated_hash, computed_hash);
        }
    }
}
