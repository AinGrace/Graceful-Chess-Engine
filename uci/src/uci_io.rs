use std::{
    fmt::Display,
    io::{BufRead, Write},
};

use tracing::info;

use crate::uci_command::Command;

pub fn read_uci_command(reader: impl BufRead) -> Result<Option<Command>, String> {
    match reader.lines().next() {
        Some(Ok(line)) if line.is_empty() => Ok(None),
        Some(Ok(line)) => {
            info!("<- {line}");
            Ok(Some(line.parse()?))
        }
        Some(Err(_e)) => Err("failed to read UCI command".into()),
        None => Ok(Some(Command::Quit)), // EOF is treated as quit command
    }
}

pub fn send(item: impl Display, mut writer: impl Write) -> Result<(), String> {
    writeln!(writer, "{item}").map_err(|_e| "failed to send ".to_string())?;
    writer.flush().map_err(|_e| "failed to send ".to_string())?;

    Ok(())
}

pub fn send_slice<T, K>(items: T, mut writer: impl Write) -> Result<(), String>
where
    T: IntoIterator<Item = K>,
    K: Display,
{
    for item in items {
        send(item, &mut writer)?;
    }

    Ok(())
}
