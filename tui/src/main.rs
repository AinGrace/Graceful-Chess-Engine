use std::{
    io::{self, Write, stdout},
    thread,
};

use arboard::Clipboard;
use color_eyre::eyre::{Ok, Result, bail};
use ratatui::{
    DefaultTerminal,
    crossterm::{self, ExecutableCommand, event, execute},
    macros::ratatui_core::terminal,
};

use crate::{model::Model, ui::global_render, update::handle_event};

/// Model
mod model;

/// Render
mod ui;

/// Update
mod update;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    stdout().execute(event::EnableMouseCapture)?;
    ratatui::run(|term| app(term))?;

    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
    let mut model = Model::new();

    while !model.should_exit() {
        assert_size(terminal.size()?)?;
        let _completed_frame = terminal.draw(|f| ui::global_render(f, &mut model))?;

        let message = update::handle_event()?;

        match message {
            Some(msg) => update::update(&mut model, msg),
            None => continue,
        }
    }

    Ok(())
}

fn assert_size(size: ratatui::prelude::Size) -> color_eyre::Result<()> {
    if size.height < 30 {
        bail!("term height is too small")
    }

    Ok(())
}

fn copy_to_clipboard(text: &str) -> color_eyre::Result<()> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;

    Ok(())
}
