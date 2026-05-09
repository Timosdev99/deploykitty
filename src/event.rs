use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::app::{AppState, Focus, MENU_ITEMS, SCROLL_PAGE_SIZE, SIDEBAR_W, sidebar_hit_test};

pub fn handle_events(app: &mut AppState) -> Result<bool> {
    if event::poll(Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if handle_form_key(app, key.code) {
                    return Ok(false);
                }
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(true);
                    }
                    KeyCode::Esc => {
                        if app.profile_form.active {
                            app.profile_form.active = false;
                        } else {
                            return Ok(true);
                        }
                    }
                    KeyCode::Tab => {
                        if app.profile_form.active && app.focus == Focus::Content {
                            let next = (app.profile_form.cursor + 1) % app.profile_form.field_count();
                            app.profile_form.cursor = next;
                        } else {
                            app.focus = match app.focus {
                                Focus::Sidebar => Focus::Content,
                                Focus::Content => Focus::Input,
                                Focus::Input => Focus::Logs,
                                Focus::Logs => Focus::Sidebar,
                            };
                        }
                    }
                    KeyCode::Up => match app.focus {
                        Focus::Sidebar if app.selected_menu > 0 => {
                            app.selected_menu -= 1;
                            app.content_cursor = 0;
                        }
                        Focus::Content => {
                            if app.profile_form.active {
                                let c = app.profile_form.cursor;
                                if c > 0 {
                                    app.profile_form.cursor = c - 1;
                                }
                            } else if app.content_cursor > 0 {
                                app.content_cursor -= 1;
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Down => match app.focus {
                        Focus::Sidebar if app.selected_menu + 1 < MENU_ITEMS.len() => {
                            app.selected_menu += 1;
                            app.content_cursor = 0;
                        }
                        Focus::Content => {
                            if app.profile_form.active {
                                let c = app.profile_form.cursor;
                                let max = app.profile_form.field_count() - 1;
                                if c < max {
                                    app.profile_form.cursor = c + 1;
                                }
                            } else {
                                let max = match app.selected_menu {
                                    0 => app.deploy_targets.len().saturating_sub(1),
                                    2 => app.profiles.len().saturating_sub(1),
                                    _ => 0,
                                };
                                if app.content_cursor < max {
                                    app.content_cursor += 1;
                                }
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Enter => match app.focus {
                        Focus::Sidebar => {
                            app.focus = Focus::Content;
                        }
                        Focus::Input => {
                            app.submit_input();
                        }
                        Focus::Content => {
                            if app.profile_form.active {
                                if app.profile_form.cursor == 5 {
                                    app.save_profile_form();
                                }
                            } else if app.selected_menu == 2 {
                                let i = app.content_cursor;
                                if i < app.profiles.len() {
                                    app.connect_to(i);
                                }
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Char(' ') => {
                        if app.focus == Focus::Content
                            && !app.profile_form.active
                            && app.selected_menu == 0
                        {
                            let i = app.content_cursor;
                            if i < app.deploy_targets.len() {
                                app.deploy_targets[i].selected = !app.deploy_targets[i].selected;
                            }
                        } else if app.focus == Focus::Input {
                            app.input.push(' ');
                        }
                    }
                    KeyCode::Char('d' | 'D') => {
                        if app.focus == Focus::Content
                            && !app.profile_form.active
                            && app.selected_menu == 0
                        {
                            app.start_deployment();
                        }
                    }
                    KeyCode::Char('a' | 'A') => {
                        if app.focus == Focus::Content
                            && !app.profile_form.active
                            && app.selected_menu == 2
                        {
                            app.open_profile_form();
                        }
                    }
                    KeyCode::PageUp => match app.focus {
                        Focus::Sidebar | Focus::Content | Focus::Input => {
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
                        Focus::Sidebar | Focus::Content | Focus::Input => {
                            app.set_scroll_offset(
                                app.scroll_offset().saturating_add(SCROLL_PAGE_SIZE),
                            );
                        }
                        Focus::Logs => {
                            app.logs_scroll_offset =
                                app.logs_scroll_offset.saturating_add(SCROLL_PAGE_SIZE);
                        }
                    },
                    KeyCode::End => {
                        app.following_chat = true;
                    }
                    KeyCode::Backspace => {
                        if app.focus == Focus::Input && !app.profile_form.active {
                            app.input.pop();
                        }
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let col = mouse.column;
                    let row = mouse.row;

                    if col < SIDEBAR_W {
                        if let Some(i) = sidebar_hit_test(row) {
                            app.selected_menu = i;
                            app.content_cursor = 0;
                            app.focus = Focus::Sidebar;
                        }
                    } else {
                        app.focus = Focus::Content;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

fn handle_form_key(app: &mut AppState, code: KeyCode) -> bool {
    if !app.profile_form.active || app.focus != Focus::Content {
        return false;
    }
    let cursor = app.profile_form.cursor;
    if cursor >= 5 {
        return false;
    }
    match code {
        KeyCode::Char(ch) => {
            if let Some(field) = app.profile_form.field_mut(cursor) {
                field.push(ch);
            }
            true
        }
        KeyCode::Backspace => {
            if let Some(field) = app.profile_form.field_mut(cursor) {
                field.pop();
            }
            true
        }
        _ => false,
    }
}
