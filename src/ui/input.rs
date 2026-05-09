use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::Focus;
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, input: &str, focus: Focus) {
    let border = if focus == Focus::Input {
        theme::SECONDARY
    } else {
        theme::BORDER
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border));

    let cursor_visible = if focus == Focus::Input { "█" } else { "" };

    let display = if input.is_empty() {
        " type a message or /help..."
    } else {
        input
    };

    let prompt = Line::from(vec![
        Span::styled(" > ", Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(display, Style::default().fg(theme::FG)),
        Span::styled(cursor_visible, Style::default().fg(theme::SECONDARY)),
    ]);

    let paragraph = Paragraph::new(prompt)
        .style(theme::base())
        .block(block);

    frame.render_widget(paragraph, area);
}
