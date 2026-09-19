use std::fmt::Write;

use crossterm::event::{self, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListDirection, ListItem, ListState, Paragraph,
};
use ratatui::{Frame, Terminal};

use crate::fs_ops::dir::{FsEntry, FsEntryType};
use crate::ui::state::{AppState, LogLevel, MenuMode, SelectedMenu, SubMenuState};

pub mod state;

/// Render the UI with various lists.
pub fn render(frame: &mut Frame, app_state: &mut AppState) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(5),
    ])
    .spacing(1);

    let [top, middle, bottom] = frame.area().layout(&vertical);

    let middle_horizontal =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).spacing(1);

    let [middle_left, middle_right] = middle.layout(&middle_horizontal);

    let title = Line::from_iter([
        Span::from("Backy Upy").bold(),
        Span::from(" (Press 'q' to quit and arrow keys to navigate)"),
    ]);

    frame.render_widget(title.centered(), top);

    render_left_list(
        frame,
        middle_left,
        &mut app_state.left,
        app_state.selected_state == SelectedMenu::Left,
    );
    render_right_list(
        frame,
        middle_right,
        &mut app_state.right,
        app_state.selected_state == SelectedMenu::Right,
    );
    render_bottom_list(
        frame,
        bottom,
        app_state.selected_state == SelectedMenu::Bottom,
        &mut app_state.bot,
    );
}

pub fn render_left_list(
    frame: &mut Frame,
    area: Rect,
    app_state: &mut SubMenuState,
    is_selected: bool,
) {
    render_dir_list(frame, area, app_state, is_selected, "Source");
}
pub fn render_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    const BASE: f64 = 1000.0;

    let mut value = size as f64;
    let mut unit = 0;

    while value >= BASE && unit < UNITS.len() - 1 {
        value /= BASE;
        unit += 1;
    }

    if unit == 0 {
        format!("{}{}", size, UNITS[unit])
    } else {
        format!("{:.1}{}", value, UNITS[unit])
    }
}

pub fn render_mount_list(
    frame: &mut Frame,
    area: Rect,
    app_state: &mut SubMenuState,
    is_selected: bool,
) {
    let items: Vec<ListItem> = app_state
        .mount_list
        .curr_list
        .iter()
        .map(|x| {
            ListItem::new(format!(
                "{} | {:?} | {}",
                &x.dev,
                &x.mounted_to,
                render_size(x.total_space)
            ))
        })
        .collect();

    let b = Block::default()
        .title(Line::from("Choose Destination Mount").left_aligned())
        .title(Line::from(app_state.dir_list.curr_path.clone()).right_aligned())
        .borders(Borders::ALL)
        .border_type(if is_selected {
            BorderType::HeavyDoubleDashed
        } else {
            BorderType::Plain
        });

    if !items.is_empty() {
        let list = List::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .block(b);
        frame.render_stateful_widget(list, area, &mut app_state.mount_list.list_state);
    } else {
        let p = Paragraph::new("No mounts found").bold().centered().block(b);
        frame.render_widget(p, area);
    }
}

fn file_name(entry: &FsEntry) -> String {
    let icon: String = match entry.e_type {
        FsEntryType::Folder => "🗀".into(),
        _ => String::new(),
    };
    format!("{} {}", icon, entry.name)
}

pub fn render_dir_list(
    frame: &mut Frame,
    area: Rect,
    app_state: &mut SubMenuState,
    is_selected: bool,
    title: &str,
) {
    let items: Vec<ListItem> = app_state
        .dir_list
        .curr_list
        .iter()
        .map(|x| {
            ListItem::new(file_name(x)).style(if x.ignored {
                Color::Yellow
            } else {
                Color::Reset
            })
        })
        .collect();

    let b = Block::default()
        .title(Line::from(title).left_aligned())
        .title(Line::from(app_state.dir_list.curr_path.clone()).right_aligned())
        .borders(Borders::ALL)
        .border_type(if is_selected {
            BorderType::HeavyDoubleDashed
        } else {
            BorderType::Plain
        });
    if !items.is_empty() {
        let list = List::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .block(b);

        frame.render_stateful_widget(list, area, &mut app_state.dir_list.list_state);
    } else {
        let p = Paragraph::new("Dir Empty").bold().centered().block(b);
        frame.render_widget(p, area);
    }
}

pub fn render_right_list(
    frame: &mut Frame,
    area: Rect,
    app_state: &mut SubMenuState,
    is_selected: bool,
) {
    if app_state.menu_mode == MenuMode::Mount {
        render_mount_list(frame, area, app_state, is_selected);
    } else {
        render_dir_list(frame, area, app_state, is_selected, "Destination");
    }
}

pub fn render_bottom_list(
    frame: &mut Frame,
    area: Rect,
    is_selected: bool,
    state: &mut SubMenuState,
) {
    let block = Block::new()
        .title(
            "Use ◄ ► to change tab, ▲ ▼  to scroll, Enter to change change dir, Escape to go back, Space to ignore, `c` to confirm",
        )
        .borders(Borders::TOP)
        .border_type(if is_selected {
            BorderType::HeavyDoubleDashed
        } else {
            BorderType::Plain
        });
    let items: Vec<ListItem> = state
        .log_list
        .logs
        .iter()
        .map(|x| {
            ListItem::new(x.message.clone()).style(match x.level {
                LogLevel::Warn => Color::Yellow,
                LogLevel::Error => Color::Red,
                _ => Color::Reset,
            })
        })
        .collect();

    let list = List::new(items)
        .style(Color::White)
        .highlight_style(Style::new().yellow().italic())
        .highlight_symbol("> ".red())
        .scroll_padding(1)
        .direction(ListDirection::BottomToTop)
        .repeat_highlight_symbol(true)
        .block(block);
    frame.render_stateful_widget(list, area, &mut state.log_list.list_state);
}
