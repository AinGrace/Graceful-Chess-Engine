use std::{env::current_dir, fs::OpenOptions, path::Path};

use tracing_subscriber::fmt;

use crate::uci::Uci;

mod uci;
mod uci_command;
mod uci_io;

fn main() {
    init_log();

    let mut uci = Uci::new_stdio();
    uci.run();
}

fn init_log() {
    let log_dir = option_env!("LOG_DIR");
    let engine_meta = option_env!("ENGINE_META");

    let current_dir = current_dir()
        .expect("unable to get current directory")
        .to_string_lossy()
        .into_owned();

    let log_dir = log_dir.unwrap_or(&current_dir);
    let log_filename = format!("{}.log", engine_meta.unwrap_or("uci"));

    let log_path = Path::new(log_dir).join(log_filename);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|_| panic!("unable to open log file at {}", log_path.display()));

    fmt().with_writer(file).init();
}
