use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{AppState, ConnectionState, MENU_ITEMS};
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    let btn_w = inner.width.saturating_sub(2) as usize;
    let selected = app.selected_menu;

    let mut lines: Vec<Line> = Vec::new();

    // Button rows: y=2..(2+N*4+3). Keep in sync with SIDEBAR_ITEM_Y0/STRIDE in app.rs.
    lines.push(Line::from(Span::styled(
        "  MENU",
        Style::default()
            .fg(theme::COMMENT)
            .add_modifier(Modifier::BOLD)
            .bg(theme::SIDEBAR_BG),
    )));
    lines.push(Line::from(""));

    for (i, item) in MENU_ITEMS.iter().enumerate() {
        let is_active = i == selected;

        let border_style = if is_active {
            Style::default().fg(theme::BTN_BORDER_ACTIVE).bg(theme::BTN_BG)
        } else {
            Style::default().fg(theme::BTN_BORDER).bg(theme::BTN_BG)
        };

        lines.push(Line::from(Span::styled(
            format!(" ╭─{}─╮ ", "─".repeat(btn_w.saturating_sub(4))),
            border_style,
        )));

        let label = item.to_string();
        let pad = btn_w.saturating_sub(label.len());
        let left_pad = pad / 2;
        let right_pad = pad - left_pad;

        let text_style = if is_active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
                .bg(theme::BTN_BG)
        } else {
            Style::default().fg(theme::BTN_TEXT).bg(theme::BTN_BG)
        };

        let arrow = if is_active { "▶" } else { " " };

        lines.push(Line::from(Span::styled(
            format!(
                " {}{}{}{} ",
                " ".repeat(left_pad),
                arrow,
                label,
                " ".repeat(right_pad.saturating_sub(1)),
            ),
            text_style,
        )));

        lines.push(Line::from(Span::styled(
            format!(
                " ╰{}╯ ",
                "─".repeat(btn_w.saturating_sub(2))
            ),
            border_style,
        )));

        lines.push(Line::from(Span::styled(
            " ",
            Style::default().bg(theme::SIDEBAR_BG),
        )));
    }

    lines.push(Line::from(Span::styled(
        "  Tab: cycle focus",
        Style::default()
            .fg(theme::COMMENT)
            .bg(theme::SIDEBAR_BG)
            .add_modifier(Modifier::ITALIC),
    )));

    let (status_icon, status_color) = match app.connection_state {
        ConnectionState::Disconnected => ("\u{25cb}", theme::COMMENT),
        ConnectionState::Connecting => ("\u{25d0}", theme::YELLOW),
        ConnectionState::Connected => ("\u{25cf}", theme::GREEN),
        ConnectionState::Failed => ("\u{2717}", theme::PRIMARY),
    };
    lines.push(Line::from(Span::styled(
        format!("  {} SSH", status_icon),
        Style::default()
            .fg(status_color)
            .bg(theme::SIDEBAR_BG)
            .add_modifier(Modifier::BOLD),
    )));

    let paragraph = Paragraph::new(lines)
        .style(theme::sidebar_base())
        .block(block);
    frame.render_widget(paragraph, area);
}
