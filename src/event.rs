use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::app::{AppState, Focus, MENU_ITEMS, SCROLL_PAGE_SIZE, SIDEBAR_W, sidebar_hit_test};

pub fn handle_events(app: &mut AppState) -> Result<bool> {
    if event::poll(Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(true);
                }
                KeyCode::Esc => return Ok(true),
                KeyCode::Tab => {
                    app.focus = match app.focus {
                        Focus::Sidebar => Focus::Input,
                        Focus::Input => Focus::Logs,
                        Focus::Logs => Focus::Sidebar,
                    };
                }
                KeyCode::Up => {
                    if app.focus == Focus::Sidebar && app.selected_menu > 0 {
                        app.selected_menu -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.focus == Focus::Sidebar
                        && app.selected_menu + 1 < MENU_ITEMS.len()
                    {
                        app.selected_menu += 1;
                    }
                }
                KeyCode::Enter => match app.focus {
                    Focus::Sidebar => {
                        app.focus = Focus::Input;
                    }
                    Focus::Input => {
                        app.submit_input();
                    }
                    Focus::Logs => {}
                },
                KeyCode::PageUp => match app.focus {
                    Focus::Sidebar | Focus::Input => {
                        let offset = app.scroll_offset();
                        app.set_scroll_offset(offset.saturating_sub(SCROLL_PAGE_SIZE));
                        app.following_chat = false;
                    }
                    Focus::Logs => {
                        app.logs_scroll_offset =
                            app.logs_scroll_offset.saturating_sub(SCROLL_PAGE_SIZE);
                    }
                },
                KeyCode::PageDown => match app.focus {
                    Focus::Sidebar | Focus::Input => {
                        app.set_scroll_offset(app.scroll_offset().saturating_add(SCROLL_PAGE_SIZE));
                    }
                    Focus::Logs => {
                        app.logs_scroll_offset =
                            app.logs_scroll_offset.saturating_add(SCROLL_PAGE_SIZE);
                    }
                },
                KeyCode::End => {
                    app.following_chat = true;
                }
                KeyCode::Char(ch) => {
                    if app.focus == Focus::Input {
                        app.input.push(ch);
                    }
                }
                KeyCode::Backspace => {
                    if app.focus == Focus::Input {
                        app.input.pop();
                    }
                }
                _ => {}
            },
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let col = mouse.column;
                    let row = mouse.row;

                    if col < SIDEBAR_W {
                        if let Some(i) = sidebar_hit_test(row) {
                            app.selected_menu = i;
                            app.focus = Focus::Sidebar;
                        }
                    } else {
                        app.focus = Focus::Input;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(false)
}
