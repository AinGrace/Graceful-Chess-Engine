use std::{
    env::current_dir,
    fmt::Display,
    fs::OpenOptions,
    io::{self, BufRead, Write, stdin},
    mem,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Instant,
};

use engine::{
    eval::{self, Score},
    search,
};
use position::{fen::Fen, position::Position};
use std::format as fmt;
use tracing::info;
use tracing_subscriber::fmt;

static ENGINE_NAME: &str = "Graceful";
static AUTHOR: &str = "AinGrace";

fn main() {
    init_log();

    let stdin = stdin();
    let mut stop_flag = Arc::new(AtomicBool::new(false));
    let mut pos = Position::new();

    for line in stdin.lock().lines() {
        let line = line.expect(
            "encountered invalid UTF-8 or something else while trying to parse stdin for uci",
        );

        info!("received: {line}");

        match line.as_str() {
            "uci" => {
                send_slice([
                    fmt!("id name {ENGINE_NAME}"),
                    fmt!("id author {AUTHOR}"),
                    fmt!("uciok"),
                ]);
            }

            "isready" => {
                send(fmt!("readyok"));
            }

            "ucinewgame" => {
                pos = Position::new();
                stop_flag = Arc::new(AtomicBool::new(false));
            }

            "quit" => {
                break;
            }

            "eval" => {
                send(fmt!("{}", eval::static_eval(&pos)));
            }

            "d" => {
                send_slice([
                    fmt!("{:#?}", pos),
                    fmt!("{}", pos.into_fen().to_string()),
                    fmt!("legal moves: {:?}", pos.legal_moves()),
                ]);
            }

            "stop" => {
                stop_flag.swap(true, Ordering::Relaxed);
            }

            _ => {
                if line.starts_with("position") {
                    handle_position(&mut pos, &line);
                }

                if line.starts_with("go") {
                    handle_go(&mut pos, &line, &stop_flag);
                }
            }
        }
    }
}

fn init_log() {
    let log_dir = option_env!("LOG_DIR");
    let engine_meta = option_env!("ENGINE_META");

    let current_dir = current_dir()
        .expect("unable to get current directory")
        .to_string_lossy()
        .into_owned();

    let log_dir = log_dir.unwrap_or(&current_dir);
    let log_filename = format!("{}.log", engine_meta.unwrap_or("engine"));

    let log_path = Path::new(log_dir).join(log_filename);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect(&format!("unable to open log file at {log_path:?}"));

    fmt().with_writer(file).init();
}

fn handle_go(pos: &mut Position, line: &str, stop_flag: &Arc<AtomicBool>) {
    let mut commands = line.split_whitespace();

    let _ = commands.nth(0);

    for sub_cmd in commands {
        match sub_cmd {
            "infinite" => {
                info!("received unsupported command [infinite]")
            }
            _ => (),
        }
    }

    let mut pos = pos.clone();
    let stop_flag = Arc::clone(stop_flag);

    thread::spawn(move || {
        info!("starting search");
        let before_search = Instant::now();
        let (score, maybe_best_move) =
            search::negamax(&mut pos, 4, Score::Mate(-1), Score::Mate(1), &stop_flag);
        let duration = Instant::now().duration_since(before_search);
        info!("search finished in: {} micros", duration.as_micros());

        if let Some(best_move) = maybe_best_move {
            send_slice([
                fmt!("info {score}"),
                fmt!("bestmove {}", best_move.to_uci()),
            ]);
        } else {
            send(fmt!("bestmove 0000"));
        }

        stop_flag.swap(false, Ordering::Relaxed);
    });
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
            info!("pos after the moves {pos:#?}");
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

fn send(s: impl Display) {
    println!("{s}");
    io::stdout().flush().expect("can't flush to stdout");
    info!("sent [{s}]")
}

fn send_slice<T, K>(items: T)
where
    T: IntoIterator<Item = K>,
    K: Display,
{
    for item in items {
        println!("{item}");
        io::stdout().flush().expect("can't flush to stdout");
        info!("sent [{item}]")
    }
}
