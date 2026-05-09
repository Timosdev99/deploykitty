use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{AppState, Focus};
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let border = if app.focus == Focus::Logs {
        theme::GREEN
    } else {
        theme::BORDER
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            " LOGS ",
            Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(ratatui::layout::Alignment::Left);

    let lines: Vec<Line> = app
        .logs
        .iter()
        .map(|line| {
            Line::from(vec![
                Span::styled(" \u{2022} ", Style::default().fg(theme::COMMENT)),
                Span::styled(line.clone(), Style::default().fg(theme::FG)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .style(theme::base())
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
