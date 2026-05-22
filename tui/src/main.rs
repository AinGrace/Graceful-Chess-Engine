use color_eyre::eyre::{Ok, Result, bail};
use ratatui::{DefaultTerminal, macros::ratatui_core::terminal};

use crate::{model::Model, ui::global_render, update::handle_event};

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

fn assert_size(size: ratatui::prelude::Size) -> Result<()> {
    if size.height < 3 {
        bail!("term height is too small")
    }

    Ok(())
}
