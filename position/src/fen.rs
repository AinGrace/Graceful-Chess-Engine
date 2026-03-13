use std::{
    error::Error,
    fmt::{Debug, Display},
    num::NonZeroU32,
    str::FromStr,
};

use types::{
    castlings::Castlings, color::Color, file::File, piece::Piece, rank::Rank, square::Square,
};

use crate::{
    board::Board,
    chessboard::{ChessBoard, PositionError},
};

// TODO make enum
// TODO iterate over bytes
#[derive(Debug)]
pub struct FenError(String);

impl Error for FenError {}

impl Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

/// A struct parsed from FEN string, modeling the position which may not be legal
#[derive(Debug, Clone)]
pub struct Fen {
    pub board: Board,
    pub turn: Color,
    pub castlings: Castlings,
    pub ep_square: Option<Square>,
    pub half_moves: u32,
    pub full_moves: NonZeroU32,
}

impl Fen {
    pub fn new(raw_fen: &str) -> Result<Self, FenError> {
        let maybe_space = raw_fen.find(' ');
        let Some(space) = maybe_space else {
            return Err(FenError("No space character".into()));
        };

        let (piece_layout, position_data) = raw_fen.split_at(space);

        let board = parse_board(piece_layout)?;
        let (turn, castling_rights, ep_square, half_moves, full_moves) =
            parse_positional_data(position_data)?;

        Ok(Self {
            board,
            turn,
            castlings: castling_rights,
            ep_square,
            half_moves,
            full_moves,
        })
    }

    pub fn into_chessboard(self) -> Result<ChessBoard, PositionError> {
        ChessBoard::from_fen(self)
    }

    fn board_to_fen_position_setup(&self) -> String {
        let mut buffer = String::with_capacity(64);

        for rank in (0..8).rev() {
            let mut empty_sqr = 0;

            for file in 0..8 {
                let square = Square::from_u32_checked(rank * 8 + file);

                match self.board.peek(square) {
                    Some(piece) => {
                        if empty_sqr != 0 {
                            buffer.push(char::from_digit(empty_sqr, 10).unwrap());
                            empty_sqr = 0;
                        }

                        buffer.push(piece.char());
                    }
                    None => empty_sqr += 1,
                }
            }

            if empty_sqr != 0 {
                buffer.push(char::from_digit(empty_sqr, 10).unwrap());
            }

            if rank != 0 {
                buffer.push('/');
            }
        }

        buffer
    }
}

impl Display for Fen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.board_to_fen_position_setup(),
            self.turn.char(),
            self.castlings,
            self.ep_square.map_or("-".into(), |ep| ep.to_string()),
            self.half_moves,
            self.full_moves
        )
    }
}

impl FromStr for Fen {
    type Err = FenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

fn parse_board(piece_layout: &str) -> Result<Board, FenError> {
    let mut board = Board::new_empty();

    let mut rank = 7;
    let mut file = 0;

    for chr in piece_layout.chars() {
        match chr {
            '/' => {
                if file != 8 {
                    return Err(FenError("Rank does not contain 8 squares".into()));
                }

                rank -= 1;
                file = 0;
            }

            '1'..='8' => {
                file += chr.to_digit(10).unwrap() as i32;
            }

            raw_piece => {
                let piece = parse_piece(raw_piece)?;

                if file > 7 {
                    return Err(FenError("Too many files in a rank".into()));
                }

                let sqr = parse_square(file, rank);

                board.set_piece_at(piece, sqr);

                file += 1;
            }
        }

        if rank < 0 {
            return Err(FenError("Too many ranks".into()));
        }
    }

    Ok(board)
}

fn parse_piece(chr: char) -> Result<Piece, FenError> {
    Piece::from_char(chr).ok_or(FenError("Invalid piece char".into()))
}

fn parse_square(file: i32, rank: i32) -> Square {
    let file = File::from_u32_checked(file as u32);
    let rank = Rank::from_u32_checked(rank as u32);

    Square::of(file, rank)
}

fn parse_positional_data(
    position_data: &str,
) -> Result<(Color, Castlings, Option<Square>, u32, NonZeroU32), FenError> {
    let mut chunks = position_data.split_whitespace();

    let side_to_move = parse_side_to_move(&mut chunks)?;
    let rights = parse_castling_rights(&mut chunks)?;
    let ep_sqr = parse_ep_square(&mut chunks)?;
    let (half_moves, full_moves) = parse_half_and_full_moves(&mut chunks)?;

    Ok((side_to_move, rights, ep_sqr, half_moves, full_moves))
}

fn parse_side_to_move<'a>(chunks: &mut impl Iterator<Item = &'a str>) -> Result<Color, FenError> {
    let first_chunk = chunks
        .next()
        .ok_or(FenError("Side to move missing".into()))?;

    let side_to_move_char = first_chunk.chars().next().ok_or(FenError(
        "Side to move should contain a singular character".into(),
    ))?;

    let side_to_move = Color::from_char(side_to_move_char).ok_or(FenError(
        "Invalid side to move char, should be either w or b".into(),
    ))?;

    Ok(side_to_move)
}

fn parse_castling_rights<'a>(
    chunks: &mut impl Iterator<Item = &'a str>,
) -> Result<Castlings, FenError> {
    let second_chunk = chunks
        .next()
        .ok_or(FenError("Castling data is missing".into()))?;

    let rights = if second_chunk == "-" {
        Castlings::new_empty()
    } else {
        Castlings::from_str(second_chunk).ok_or(FenError("Invalid castling data".into()))?
    };

    Ok(rights)
}

fn parse_ep_square<'a>(
    chunks: &mut impl Iterator<Item = &'a str>,
) -> Result<Option<Square>, FenError> {
    let third_chunk = chunks
        .next()
        .ok_or(FenError("En passant data is missing".into()))?;

    let mut chars = third_chunk.chars();

    let ep_sqr = if third_chunk == "-" {
        None
    } else {
        let file = File::from_char(chars.next().ok_or(FenError("Invalid ep data".into()))?)
            .ok_or(FenError("Invalid ep file".into()))?;

        let rank = Rank::from_char(chars.next().ok_or(FenError("Invalid ep data".into()))?)
            .ok_or(FenError("Invalid ep rank".into()))?;

        if rank != Rank::Third || rank != Rank::Sixth {
            return Err(FenError("Invalid ep rank".into()));
        }

        Some(Square::of(file, rank))
    };

    Ok(ep_sqr)
}

fn parse_half_and_full_moves<'a>(
    chunks: &mut impl Iterator<Item = &'a str>,
) -> Result<(u32, std::num::NonZero<u32>), FenError> {
    let fourth_chunk = chunks.next().ok_or(FenError("Missing half moves".into()))?;

    let half_moves = fourth_chunk
        .parse()
        .map_err(|_| FenError("Expected valid number for half moves".into()))?;

    let fifth_chunk = chunks.next().ok_or(FenError("Missing full moves".into()))?;

    let full_moves = fifth_chunk
        .parse()
        .map_err(|_| FenError("Expected valid number for full moves".into()))?;

    Ok((half_moves, full_moves))
}
