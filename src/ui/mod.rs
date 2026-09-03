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

use crate::ui::state::{AppState, SelectedMenu, SubMenuState};

pub mod state;

/// Render the UI with various lists.
pub fn render(frame: &mut Frame, app_state: &mut AppState) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
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
    );
}

pub fn render_left_list(
    frame: &mut Frame,
    area: Rect,
    app_state: &mut SubMenuState,
    is_selected: bool,
) {
    let items: Vec<ListItem> = app_state
        .dir_list
        .curr_list
        .iter()
        .map(|x| ListItem::new(x.name.clone()))
        .collect();

    let b = Block::default()
        .title(Line::from("Source").left_aligned())
        .title(Line::from(app_state.dir_list.curr_path.clone()).right_aligned())
        .borders(Borders::ALL)
        .border_type(if is_selected {
            BorderType::HeavyDoubleDashed
        } else {
            BorderType::Plain
        });
    let list = List::new(items)
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ")
        .block(b);

    frame.render_stateful_widget(list, area, &mut app_state.dir_list.list_state);
}

pub fn render_right_list(
    frame: &mut Frame,
    area: Rect,
    app_state: &mut SubMenuState,
    is_selected: bool,
) {
    let items: Vec<ListItem> = app_state
        .dir_list
        .curr_list
        .iter()
        .map(|x| ListItem::new(x.name.clone()))
        .collect();

    let b = Block::default()
        .title(Line::from("Destination").left_aligned())
        .title(Line::from(app_state.dir_list.curr_path.clone()).right_aligned())
        .borders(Borders::ALL)
        .border_type(if is_selected {
            BorderType::HeavyDoubleDashed
        } else {
            BorderType::Plain
        });

    let list = List::new(items)
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ")
        .block(b);

    frame.render_stateful_widget(list, area, &mut app_state.dir_list.list_state);
}

pub fn render_bottom_list(frame: &mut Frame, area: Rect, is_selected: bool) {
    let block = Block::new()
        .title(
            " Use ◄ ► to change tab, ▲ ▼  to scroll, Enter to change change dir, Escape to go back",
        )
        .borders(Borders::TOP)
        .border_type(if is_selected {
            BorderType::HeavyDoubleDashed
        } else {
            BorderType::Plain
        });
    frame.render_widget(block, area);
}
