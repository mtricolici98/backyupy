use std::path::Path;

use ratatui::widgets::ListState;

use crate::fs_ops::{
    dir::{FsEntry, FsEntryType, list_dir_own, mark_for_ignore},
    mounts::{Mount, get_avail_mounts},
};

#[derive(Default, PartialEq)]
pub enum MenuMode {
    #[default]
    Mount,
    Dir,
}

pub trait MoveUpDown {
    fn move_up(&mut self);

    fn move_down(&mut self);
}

#[derive(Default)]
pub struct DirListState {
    pub curr_path: String,
    pub curr_list: Vec<FsEntry>,
    pub list_state: ListState,
    pub source_path_stack: Vec<(usize, String)>,
}

impl DirListState {
    fn new(path: Option<String>) -> DirListState {
        let mut instance = DirListState::default();
        if let Some(path) = path {
            instance.curr_list =
                list_dir_own(&Path::new(&path)).expect("Start path should be accessible");
            instance.curr_path = path;
            instance.list_state.select(Some(0));
        }
        instance
    }

    fn move_dir(&mut self) -> anyhow::Result<()> {
        let selected = self.list_state.selected();
        let Some(selected_idx) = selected else {
            return Ok(());
        };
        let Some(selected_item) = self.curr_list.get(selected_idx) else {
            return Ok(());
        };
        if selected_item.e_type == FsEntryType::Folder {
            let path = selected_item.abs_path.clone();
            let new_list = list_dir_own(Path::new(&path))?;
            self.curr_list = new_list;
            self.source_path_stack
                .push((selected_idx, self.curr_path.clone()));
            self.curr_path = path;
            self.list_state.select(Some(0));
        }
        Ok(())
    }

    fn pop_dir(&mut self) {
        let Some((selection, back_to)) = self.source_path_stack.pop() else {
            return;
        };
        self.curr_path = back_to;
        self.list_state.select(Some(selection));
        self.curr_list = list_dir_own(Path::new(&self.curr_path)).expect("To go into folder");
    }
}

impl MoveUpDown for DirListState {
    fn move_up(&mut self) {
        self.list_state.select_previous();
    }

    fn move_down(&mut self) {
        self.list_state.select_next();
    }
}

#[derive(Default)]
pub struct MountListState {
    pub selected_mount: Option<String>,
    pub curr_list: Vec<Mount>,
    pub list_state: ListState,
}

impl MountListState {
    pub fn new() -> Self {
        let mut instance = MountListState::default();
        instance.curr_list = get_avail_mounts();
        instance.list_state.select(Some(0));
        instance
    }
}

impl MoveUpDown for MountListState {
    fn move_up(&mut self) {
        self.list_state.select_previous();
    }

    fn move_down(&mut self) {
        self.list_state.select_next();
    }
}

#[derive(Default)]
pub struct SubMenuState {
    pub dir_list: DirListState,
    pub mount_list: MountListState,
    pub log_list: LogMenuState,
    pub selected_mount: Option<String>,
    pub menu_mode: MenuMode,
    pub mount_select_allowed: bool,
}

impl SubMenuState {
    pub fn new(start_path: Option<String>) -> Self {
        let mut instance = SubMenuState::default();
        instance.menu_mode = if start_path.is_some() {
            MenuMode::Dir
        } else {
            MenuMode::Mount
        };

        instance.mount_select_allowed = start_path.is_none();
        instance.dir_list = DirListState::new(start_path);
        instance.mount_list = MountListState::new();
        instance
    }
}

#[derive(Default, PartialEq)]
pub enum SelectedMenu {
    #[default]
    Left,
    Right,
    Bottom,
}

#[derive(Default, PartialEq)]
pub enum LogLevel {
    #[default]
    Info,
    Warn,
    Error,
}

pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Default)]
pub struct LogMenuState {
    pub logs: Vec<LogEntry>,
    pub list_state: ListState,
}

#[derive(Default)]
pub struct AppState {
    pub start_path: String,
    pub left: SubMenuState,
    pub right: SubMenuState,
    pub selected_state: SelectedMenu,
    pub bot: SubMenuState,
}

impl AppState {
    pub fn get_state_to_modify(&mut self) -> &mut SubMenuState {
        if self.selected_state == SelectedMenu::Left {
            return &mut self.left;
        } else if self.selected_state == SelectedMenu::Right {
            return &mut self.right;
        }
        return &mut self.bot;
    }

    pub fn get_list_to_move(submenu: &mut SubMenuState) -> Box<&mut dyn MoveUpDown> {
        if submenu.menu_mode == MenuMode::Mount {
            return Box::new(&mut submenu.mount_list);
        } else {
            return Box::new(&mut submenu.dir_list);
        }
    }

    pub fn handle_up(&mut self) {
        let to_modify = self.get_state_to_modify();
        let list = AppState::get_list_to_move(to_modify);
        list.move_up();
    }

    pub fn handle_down(&mut self) {
        let to_modify = self.get_state_to_modify();
        let list = AppState::get_list_to_move(to_modify);
        list.move_down();
    }

    pub fn handle_enter(&mut self) {
        let to_modify = self.get_state_to_modify();
        if to_modify.menu_mode == MenuMode::Dir {
            if let Err(e) = to_modify.dir_list.move_dir() {
                self.bot.log_list.logs.push(LogEntry {
                    level: LogLevel::Error,
                    message: format!("{:?}", e),
                });
            }
        }
    }

    pub fn handle_space(&mut self) {
        let to_modify = self.get_state_to_modify();
        if to_modify.menu_mode == MenuMode::Dir {
            let Some(selected) = to_modify.dir_list.list_state.selected() else {
                return;
            };
            let mark_for_ignore = mark_for_ignore(&mut to_modify.dir_list.curr_list, selected);
            if let Err(e) = mark_for_ignore {
                self.bot.log_list.logs.push(LogEntry {
                    level: LogLevel::Error,
                    message: format!("{:?}", e),
                });
            } else {
                self.bot.log_list.logs.push(LogEntry {
                    level: LogLevel::Info,
                    message: "Ignore list modified".to_string(),
                });
            }
        }
    }

    pub fn handle_esc(&mut self) {
        let state = self.get_state_to_modify();
        if state.menu_mode == MenuMode::Dir {
            state.dir_list.pop_dir();
        }
    }

    pub fn handle_right(&mut self) {
        if self.selected_state == SelectedMenu::Left {
            self.selected_state = SelectedMenu::Right;
            return;
        }
        if self.selected_state == SelectedMenu::Right {
            self.selected_state = SelectedMenu::Bottom;
            return;
        }
    }

    pub fn handle_left(&mut self) {
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
