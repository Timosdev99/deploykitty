use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::VERSION;
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " DeployKitty ",
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v", Style::default().fg(theme::COMMENT)),
        Span::styled(VERSION, Style::default().fg(theme::COMMENT)),
        Span::styled("  \u{2022}  ", Style::default().fg(theme::COMMENT)),
        Span::styled("Ctrl+C / Esc: quit", Style::default().fg(theme::COMMENT)),
        Span::styled("  \u{2022}  ", Style::default().fg(theme::COMMENT)),
        Span::styled("PgUp/PgDn: scroll", Style::default().fg(theme::COMMENT)),
    ]))
    .style(theme::base().fg(theme::FG))
    .block(block);

    frame.render_widget(header, area);
}
