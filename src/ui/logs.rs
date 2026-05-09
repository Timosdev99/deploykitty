use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{AppState, Focus};
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut AppState) {
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

    let inner = block.inner(area);
    let total = lines.len();
    let view_h = inner.height as usize;

    if total > view_h {
        let max_scroll = total - view_h;
        if app.logs_scroll_offset > max_scroll {
            app.logs_scroll_offset = max_scroll;
        }
    } else {
        app.logs_scroll_offset = 0;
    }

    let paragraph = Paragraph::new(lines)
        .style(theme::base())
        .scroll((app.logs_scroll_offset as u16, 0))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
