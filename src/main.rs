use clap::Parser;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use std::path::Path;

use crate::config::Args;
use crate::fs_ops::dir::{FsEntry, FsEntryType, get_config, init_config, list_dir_own};
use crate::fs_ops::mounts::Mount;
use crate::ui::render;
use crate::ui::state::{AppState, SubMenuState};

mod config;
mod fs_ops;
mod ui;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    color_eyre::install().expect("To work");
    let _ = init_config(&args.path);
    ratatui::run(|terminal| {
        let mut app_sate = AppState::default();
        app_sate.start_path = args.path.clone();
        app_sate.left = SubMenuState::new(Some(args.path.clone()));
        app_sate.right = SubMenuState::new(None);
        app_sate.run(terminal)
    })
}

impl AppState {
    fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| render(frame, &mut self))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => self.handle_down(),
                    KeyCode::Char('k') | KeyCode::Up => self.handle_up(),
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Char('h') | KeyCode::Left => self.handle_left(),
                    KeyCode::Char('l') | KeyCode::Right => self.handle_right(),
                    KeyCode::Char('s') | KeyCode::Esc => self.handle_esc(),
                    KeyCode::Char('w') | KeyCode::Enter => self.handle_enter(),
                    KeyCode::Char('e') | KeyCode::Char(' ') => self.handle_space(),
                    _ => {}
                }
            }
        }
    }
}
