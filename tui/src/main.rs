use color_eyre::eyre::Ok;
use ratatui::{DefaultTerminal, macros::ratatui_core::terminal};

use crate::{model::Model, ui::render, update::handle_event};

/// Model
mod model;

/// Render
mod ui;

/// Update
mod update;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(|term| app(term))?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
    let mut model = Model::new();

    while !model.exit {
        let _completed_frame = terminal.draw(|f| ui::render(f, &model))?;

        let message = update::handle_event()?;

        match message {
            Some(msg) => update::update(&mut model, msg),
            None => continue,
        }
    }

    Ok(())
}
