use std::{
    io::{BufRead, stdin},
    mem,
};

use engine::{eval, search};
use position::{fen::Fen, position::Position};

static ENGINE_NAME: &str = "Graceful";
static AUTHOR: &str = "AinGrace";

fn main() {
    let stdin = stdin();
    let mut pos = Position::new();

    for line in stdin.lock().lines() {
        let line = line.expect(
            "encountered invalid UTF-8 or something else while trying to parse stdin for uci",
        );

        match line.as_str() {
            "uci" => {
                println!("id name {ENGINE_NAME}");
                println!("id author {AUTHOR}");
                println!("uciok");
            }

            "isready" => {
                println!("readyok");
            }

            "ucinewgame" => {
                pos = Position::new();
            }

            "quit" => {
                break;
            }

            "eval" => {
                println!("{}", eval::static_eval(&pos));
            }

            _ => {
                if line.starts_with("position") {
                    handle_position(&mut pos, &line);
                }

                if line.starts_with("go depth") {
                    handle_go(&mut pos, &line);
                }
            }
        }
    }
}

fn handle_go(pos: &mut Position, line: &str) {
    let mut commands = line.split_whitespace();

    // drop first two words i.e "go depth"
    let _ = commands.nth(1);

    let Some(raw_depth) = commands.next() else {
        eprintln!("error trying to get the raw depth");
        return;
    };

    let Ok(depth) = raw_depth.parse() else {
        eprintln!("error trying to parse the raw depth");
        return;
    };

    let (_score, best_move) = search::negamax(pos, depth, 0);
    println!(
        "bestmove {} | score -> {_score}",
        best_move.unwrap().to_uci()
    );
}

fn handle_position(pos: &mut Position, line: &str) {
    let mut commands = line.split_whitespace();

    // drop first word
    let _ = commands.next();

    let Some(second_part) = commands.next() else {
        return;
    };

    match second_part {
        "startpos" => {
            handle_moves(pos, commands);
        }
        "fen" => {
            let Some(fen_str) = commands.next() else {
                return;
            };

            let Ok(fen) = Fen::new(fen_str) else { return };

            if let Ok(new_pos) = fen.into_position() {
                let _old_pos = mem::replace(pos, new_pos);
            } else {
                return;
            }

            handle_moves(pos, commands);
        }

        _unknown => (),
    }
}

fn handle_moves(pos: &mut Position, mut commands: std::str::SplitWhitespace<'_>) {
    if let Some(moves_cmd) = commands.next()
        && moves_cmd == "moves"
    {
        for raw_uci in commands.into_iter() {
            let Ok(_undo) = pos.uci_move(raw_uci) else {
                return;
            };
        }
    }
}
