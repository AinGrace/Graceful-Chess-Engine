use std::{fmt::Display, str::FromStr};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, Paragraph, Wrap},
};
use types::{file::File, piece::Piece, rank::Rank, square::Square};

use crate::model::{FocusMode, Model};

pub fn global_render(frame: &mut Frame, model: &mut Model) {
    let left_middle_right =
        Layout::horizontal(Constraint::from_percentages([30, 40, 30])).split(frame.area());

    let middle_top_bottom =
        Layout::vertical([Constraint::Fill(1), Constraint::Max(3), Constraint::Max(3)])
            .split(left_middle_right[1]);

    let right_top_bottom =
        Layout::vertical(Constraint::from_percentages([60, 40])).split(left_middle_right[2]);

    let log_box = build_logs(model.logs().to_vec());
    if let Some(scroll) = model.take_scrolling()
        && left_middle_right[0].contains(Position::new(scroll.col(), scroll.row()))
    {
        match scroll {
            crate::model::Scrolling::Up { .. } => model.log_state().select_previous(),
            crate::model::Scrolling::Down { .. } => model.log_state().select_next(),
        }
    }

    frame.render_stateful_widget(log_box, left_middle_right[0], model.log_state());

    let board_box = build_chessboard(model);
    frame.render_widget(board_box, middle_top_bottom[0]);

    let info_box = build_info(model);
    frame.render_widget(info_box, right_top_bottom[0]);

    let (move_input_box, fen_input_box) = build_input_boxes(model);
    frame.render_widget(move_input_box, middle_top_bottom[1]);
    frame.render_widget(fen_input_box, middle_top_bottom[2]);

    let history_box = build_history(model);
    frame.render_widget(history_box, right_top_bottom[1]);
}

fn build_history(model: &Model) -> Paragraph<'_> {
    let line = Line::from_iter(
        model
            .move_history_iter()
            .enumerate()
            .map(|(i, mv)| format!("{i}. {mv} ")),
    );

    Paragraph::new(line)
        .block(
            Block::bordered()
                .title("Move history")
                .border_type(BorderType::Rounded)
                .border_style(Style::new().light_cyan()),
        )
        .wrap(Wrap { trim: true })
}

fn build_input_boxes(model: &Model) -> (Paragraph<'static>, Paragraph<'static>) {
    let mut move_input_block = Block::bordered()
        .title("Move")
        .border_style(Style::new().yellow());

    let mut fen_input_block = Block::bordered()
        .title("Fen")
        .border_style(Style::new().yellow());

    match model.focus() {
        FocusMode::MoveInput => {
            fen_input_block = fen_input_block.border_style(Style::new().dark_gray())
        }
        FocusMode::FenInput => {
            move_input_block = move_input_block.border_style(Style::new().dark_gray())
        }
    };

    let move_input = Paragraph::new(model.partial_move_to_string())
        .block(move_input_block)
        .left_aligned();

    let fen_input = Paragraph::new(model.partial_fen_to_string())
        .block(fen_input_block)
        .left_aligned();

    (move_input, fen_input)
}

fn build_info(model: &Model) -> Paragraph<'_> {
    let line = |label: &str, val: &dyn Display| Line::from(format!("{label}: {val}"));

    let legal_moves = model
        .left_partial_move()
        .and_then(|raw| Square::from_str(&raw).ok())
        .and_then(|sq| model.legal_moves_of(sq))
        .map(|moves| {
            moves
                .iter()
                .map(|mv| mv.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "None".to_string());

    let search_time = model.search_time();

    Paragraph::new(vec![
        line("Turn", &model.turn().char()),
        line("Ep square", &model.ep_square_to_str()),
        line("Best move ", &model.best_move_to_uci()),
        line("Best move eval", &model.best_move_score().to_string()),
        line("Static Eval", &model.static_eval()),
        line("Search depth", &model.search_depth()),
        Line::from(format!(
            "Search time: millis → {} | micros → {}",
            search_time.as_millis(),
            search_time.as_micros(),
        )),
        line("Half moves", &model.half_moves()),
        line("Full moves", &model.full_moves()),
        line("Zobrist hash", &model.z_hash()),
        line("FEN", &model.position_fen()),
        Line::from(format!("Legal moves for selection: {legal_moves}")),
    ])
    .block(Block::bordered().title("Info"))
    .wrap(Wrap { trim: true })
}

// TODO: move chessboard UI building into separate module
fn build_chessboard(model: &mut Model) -> Paragraph<'static> {
    const LIGHT: Color = Color::Rgb(240, 217, 181);
    const DARK: Color = Color::Rgb(181, 136, 99);

    let partial_from = Square::from_str(&model.left_partial_move().unwrap_or_default()).ok();
    let partial_to = Square::from_str(&model.right_partial_move().unwrap_or_default()).ok();

    let mut lines: Vec<Line> = (0..8)
        .rev()
        .map(|rank| build_rank_line(model, rank, partial_from, partial_to, LIGHT, DARK))
        .collect();

    lines.push(Line::raw("    A    B    C    D    E    F    G    H"));

    Paragraph::new(lines).block(
        Block::bordered()
            .title("Graceful chess")
            .border_type(BorderType::Rounded)
            .border_style(Style::new().green()),
    )
}

fn build_rank_line(
    model: &Model,
    rank: u32,
    partial_from: Option<Square>,
    partial_to: Option<Square>,
    light: Color,
    dark: Color,
) -> Line<'static> {
    let mut spans = vec![Span::raw(format!("{} ", rank + 1))];

    for file in 0..8 {
        let square = Square::of(File::from_u32_checked(file), Rank::from_u32_checked(rank));
        let piece = model.peek_piece_at(square);
        let bg = square_background(model, square, piece, partial_from, partial_to, light, dark);

        let span = match piece {
            Some(piece) => {
                let fg = match piece.color() {
                    types::color::Color::White => Color::Rgb(255, 255, 255),
                    types::color::Color::Black => Color::Rgb(0, 0, 0),
                };
                Span::styled(
                    format!("  {}  ", piece_to_unicode(piece)),
                    Style::new().bg(bg).fg(fg),
                )
            }
            None => Span::styled("     ", Style::new().bg(bg)),
        };

        spans.push(span);
    }

    Line::from(spans)
}

fn square_background(
    model: &Model,
    square: Square,
    piece: Option<Piece>,
    partial_from: Option<Square>,
    partial_to: Option<Square>,
    light: Color,
    dark: Color,
) -> Color {
    let base = || if square.is_dark_square() { dark } else { light };

    if let Some(piece) = piece {
        return occupied_square_background(model, square, piece, partial_from, partial_to, base);
    }

    empty_square_background(model, square, partial_from, partial_to, base)
}

fn occupied_square_background(
    model: &Model,
    square: Square,
    piece: Piece,
    partial_from: Option<Square>,
    partial_to: Option<Square>,
    base: impl Fn() -> Color,
) -> Color {
    if partial_from == Some(square) {
        return if model.is_legal_origin(square) {
            Color::LightGreen
        } else {
            Color::Gray
        };
    }

    if partial_to == Some(square) {
        return if model.is_legal_dest(piece.role(), square) {
            Color::Red
        } else {
            Color::Gray
        };
    }

    base()
}

fn empty_square_background(
    model: &Model,
    square: Square,
    partial_from: Option<Square>,
    partial_to: Option<Square>,
    base: impl Fn() -> Color,
) -> Color {
    debug_assert!(
        partial_from.is_some() || partial_to.is_none(),
        "destination cannot be Some(_) without an origin"
    );

    let Some(from) = partial_from else {
        return base();
    };

    if let Some(to) = partial_to
        && square == to
    {
        return destination_square_background(model, from, to);
    }

    if model.has_legal_origin_and_destination(from, square) {
        Color::Yellow
    } else {
        base()
    }
}

fn destination_square_background(model: &Model, from: Square, to: Square) -> Color {
    let is_legal = model
        .peek_piece_at(from)
        .is_some_and(|piece| model.is_legal_dest(piece.role(), to));

    if is_legal {
        Color::LightGreen
    } else {
        Color::Gray
    }
}

fn build_logs(logs: Vec<String>) -> List<'static> {
    List::new(logs)
        .block(
            Block::bordered()
                .title("Logs")
                .border_type(BorderType::Rounded)
                .border_style(Style::new().blue()),
        )
        .highlight_symbol("=> ")
}

pub fn piece_to_unicode(piece: Piece) -> &'static str {
    match piece {
        Piece::WPawn | Piece::BPawn => "♟",
        Piece::WKnight | Piece::BKnight => "♞",
        Piece::WBishop | Piece::BBishop => "♝",
        Piece::WRook | Piece::BRook => "♜",
        Piece::WQueen | Piece::BQueen => "♛",
        Piece::WKing | Piece::BKing => "♚",
    }
}
