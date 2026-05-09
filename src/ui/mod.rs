pub mod chat;
pub mod content;
pub mod header;
pub mod input;
pub mod logs;
pub mod sidebar;

use ratatui::{Frame, layout::Layout};

use crate::app::{AppState, SIDEBAR_W};

pub fn ui(frame: &mut Frame, app: &mut AppState) {
    use ratatui::layout::{Constraint, Direction};

    let [sidebar_area, main_area] = *Layout::new(
        Direction::Horizontal,
        [Constraint::Length(SIDEBAR_W), Constraint::Min(1)],
    )
    .split(frame.area())
    .as_ref() else {
        return;
    };

    let [header_area, content_area, input_area, logs_area] = *Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(17),
        ],
    )
    .split(main_area)
    .as_ref() else {
        return;
    };

    sidebar::render(frame, sidebar_area, app);
    header::render(frame, header_area);
    content::render(frame, content_area, app);
    input::render(frame, input_area, &app.input, app.focus);
    logs::render(frame, logs_area, app);
}
