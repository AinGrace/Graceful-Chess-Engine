use std::{
    fs::OpenOptions,
    io::{self, BufRead, Write, stdin},
    mem,
};

use engine::{eval, search};
use position::{fen::Fen, position::Position};
use tracing::info;
use tracing_subscriber::fmt;

static ENGINE_NAME: &str = "Graceful";
static AUTHOR: &str = "AinGrace";

fn main() {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/home/aingrace/Graceful-Engine/bench/results/engine.log")
        .unwrap();

    fmt().with_writer(file).init();

    let stdin = stdin();
    info!("engine started");
    let mut pos = Position::new();

    for line in stdin.lock().lines() {
        let line = line.expect(
            "encountered invalid UTF-8 or something else while trying to parse stdin for uci",
        );

        match line.as_str() {
            "uci" => {
                send(format!("id name {ENGINE_NAME}"));
                send(format!("id author {AUTHOR}"));
                send(format!("uciok"));
            }

            "isready" => {
                send(format!("readyok"));
            }

            "ucinewgame" => {
                pos = Position::new();
            }

            "quit" => {
                break;
            }

            "eval" => {
                send(format!("{}", eval::static_eval(&pos)));
            }

            "d" => {
                send(format!("{:#?}", pos));
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

    // let Ok(depth) = raw_depth.parse() else {
    //     eprintln!("error trying to parse the raw depth");
    //     return;
    // };

    let (_score, maybe_best_move) = search::negamax(pos, 4);

    if let Some(best_move) = maybe_best_move {
        send(format!("info score cp {_score}"));
        send(format!("bestmove {}", best_move.to_uci()));
    } else {
        send(format!("bestmove 0000"));
    }
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
            *pos = Position::new();
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

fn send(s: String) {
    println!("{s}");
    io::stdout().flush().expect("can't flush to stdout")
}
