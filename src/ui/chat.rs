use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::app::Sender;
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);

    let mut buf: Vec<Line> = Vec::new();

    for msg in app.messages() {
        match &msg.sender {
            Sender::System => {
                buf.push(Line::from(vec![
                    Span::styled(" \u{2500}\u{2500} ", Style::default().fg(theme::COMMENT)),
                    Span::styled(
                        "system",
                        Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD),
                    ),
                ]));
                for line in msg.content.split('\n') {
                    buf.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(theme::COMMENT).add_modifier(Modifier::ITALIC),
                    )));
                }
                buf.push(Line::from(""));
            }
            Sender::User => {
                buf.push(Line::from(vec![
                    Span::styled(" \u{2500}\u{2500} ", Style::default().fg(theme::COMMENT)),
                    Span::styled(
                        "user",
                        Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD),
                    ),
                ]));
                for line in msg.content.split('\n') {
                    buf.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(theme::FG),
                    )));
                }
                buf.push(Line::from(""));
            }
            Sender::Assistant => {
                buf.push(Line::from(vec![
                    Span::styled(" \u{2500}\u{2500} ", Style::default().fg(theme::COMMENT)),
                    Span::styled(
                        "DeployKitty",
                        Style::default().fg(theme::SECONDARY).add_modifier(Modifier::BOLD),
                    ),
                ]));
                for line in msg.content.split('\n') {
                    buf.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(theme::FG),
                    )));
                }
                buf.push(Line::from(""));
            }
            Sender::Tool { name } => {
                buf.push(Line::from(vec![
                    Span::styled(" \u{2500}\u{2500} ", Style::default().fg(theme::COMMENT)),
                    Span::styled(
                        format!("tool:{}", name),
                        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                    ),
                ]));
                for line in msg.content.split('\n') {
                    buf.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(theme::COMMENT),
                    )));
                }
                buf.push(Line::from(""));
            }
        }
    }

    let total = buf.len();
    let view_h = inner.height as usize;

    if total > view_h {
        let max_scroll = total - view_h;
        if app.following_chat || app.scroll_offset() > max_scroll {
            app.set_scroll_offset(max_scroll);
        }
    }
    let scroll = app.scroll_offset().min(total.saturating_sub(1));

    let chat = Paragraph::new(buf)
        .style(theme::base())
        .scroll((scroll as u16, 0))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(chat, area);
}
