use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::app::{AppState, Focus, MENU_ITEMS, SIDEBAR_W};

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
                        Focus::Sidebar => Focus::Logs,
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
                KeyCode::Enter => {
                    if app.focus == Focus::Sidebar {
                        app.focus = Focus::Logs;
                    }
                }
                _ => {}
            },
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let col = mouse.column;
                    let row = mouse.row;

                    if col < SIDEBAR_W {
                        let item_start = 2u16;
                        for i in 0..MENU_ITEMS.len() {
                            let y0 = item_start + (i as u16) * 4;
                            let y1 = y0 + 3;
                            if row >= y0 && row <= y1 {
                                app.selected_menu = i;
                                app.focus = Focus::Sidebar;
                                break;
                            }
                        }
                    } else {
                        app.focus = Focus::Logs;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(false)
}
