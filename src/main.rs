use clap::Parser;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use std::path::Path;

use crate::config::Args;
use crate::fs_ops::{FsEntry, FsEntryType, list_dir_own};
use crate::ui::render;

mod config;
mod fs_ops;
mod ui;

#[derive(Default)]
struct SubMenuState {
    curr_path: String,
    curr_list: Vec<FsEntry>,
    source_list_state: ListState,
    source_path_stack: Vec<(usize, String)>,
    selected_mount: Option<String>,
}

impl SubMenuState {
    fn new(start_path: Option<String>) -> Self {
        let mut instance = SubMenuState::default();
        if let Some(path) = start_path {
            instance.curr_list =
                list_dir_own(&Path::new(&path)).expect("Start path should be accessible");
            instance.curr_path = path;
            instance.source_list_state.select(Some(0));
        }
        instance
    }

    fn move_dir(&mut self, selected: Option<usize>) {
        let Some(selected_idx) = selected else {
            return;
        };
        let Some(selected_item) = self.curr_list.get(selected_idx) else {
            return;
        };
        if selected_item.e_type == FsEntryType::Folder {
            self.source_path_stack
                .push((selected_idx, self.curr_path.clone()));
            self.curr_path = selected_item.abs_path.clone();
            self.source_list_state.select(Some(0));
            self.curr_list = list_dir_own(Path::new(&self.curr_path)).expect("To go into folder");
        }
    }

    fn pop_dir(&mut self) {
        let Some((selection, back_to)) = self.source_path_stack.pop() else {
            return;
        };
        self.curr_path = back_to;
        self.source_list_state.select(Some(selection));
        self.curr_list = list_dir_own(Path::new(&self.curr_path)).expect("To go into folder");
    }
}

#[derive(Default, PartialEq)]
enum SelectedMenu {
    #[default]
    Left,
    Right,
    Bottom,
}

#[derive(Default)]
struct AppState {
    start_path: String,
    left: SubMenuState,
    right: SubMenuState,
    selected_state: SelectedMenu,
}

impl AppState {
    fn get_state_to_modify(&mut self) -> &mut SubMenuState {
        if self.selected_state == SelectedMenu::Left {
            return &mut self.left;
        } else if self.selected_state == SelectedMenu::Right {
            return &mut self.right;
        }
        return &mut self.left;
    }

    fn handle_up(&mut self) {
        let to_modify = self.get_state_to_modify();
        if to_modify.source_list_state.selected().is_none() {
            to_modify.source_list_state.select(Some(0))
        } else {
            to_modify.source_list_state.select_previous();
        }
    }

    fn handle_down(&mut self) {
        let to_modify = self.get_state_to_modify();
        to_modify.source_list_state.select_next();
    }

    fn handle_enter(&mut self) {
        let to_modify = self.get_state_to_modify();
        to_modify.move_dir(to_modify.source_list_state.selected());
    }

    fn handle_esc(&mut self) {
        self.get_state_to_modify().pop_dir();
    }

    fn handle_right(&mut self) {
        if self.selected_state == SelectedMenu::Left {
            self.selected_state = SelectedMenu::Right;
            return;
        }
        if self.selected_state == SelectedMenu::Right {
            self.selected_state = SelectedMenu::Bottom;
            return;
        }
    }

    fn handle_left(&mut self) {
        if self.selected_state == SelectedMenu::Right {
            self.selected_state = SelectedMenu::Left;
            return;
        }
        if self.selected_state == SelectedMenu::Bottom {
            self.selected_state = SelectedMenu::Right;
            return;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    color_eyre::install().expect("To work");
    ratatui::run(|terminal| {
        let mut app_sate = AppState::default();
        app_sate.start_path = args.path.clone();
        app_sate.left = SubMenuState::new(Some(args.path.clone()));
        app_sate.right = SubMenuState::new(Some("/mnt/".to_string()));
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
                    KeyCode::Esc => self.handle_esc(),
                    KeyCode::Char('w') | KeyCode::Enter => self.handle_enter(),
                    _ => {}
                }
            }
        }
    }
}
