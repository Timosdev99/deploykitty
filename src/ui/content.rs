use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{AppState, ConnectionState, Focus, ProfileForm};
use crate::pipeline::Pipeline;
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut AppState) {
    if app.profile_form.active {
        render_profile_form(frame, area, app);
        return;
    }
    match app.selected_menu {
        0 => render_deploy(frame, area, app),
        2 => render_connect_vps(frame, area, app),
        _ => render_chat(frame, area, app),
    }
}

fn render_chat(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);

    let mut buf: Vec<Line> = Vec::new();
    for msg in app.messages() {
        let header = match &msg.sender {
            crate::app::Sender::System => ("system", theme::YELLOW),
            crate::app::Sender::User => ("user", theme::GREEN),
            crate::app::Sender::Assistant => ("DeployKitty", theme::SECONDARY),
            crate::app::Sender::Tool { name } => {
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
                continue;
            }
        };
        buf.push(Line::from(vec![
            Span::styled(" \u{2500}\u{2500} ", Style::default().fg(theme::COMMENT)),
            Span::styled(header.0, Style::default().fg(header.1).add_modifier(Modifier::BOLD)),
        ]));
        for line in msg.content.split('\n') {
            let fg = match &msg.sender {
                crate::app::Sender::System => theme::COMMENT,
                _ => theme::FG,
            };
            let style = match &msg.sender {
                crate::app::Sender::System => {
                    Style::default().fg(fg).add_modifier(Modifier::ITALIC)
                }
                _ => Style::default().fg(fg),
            };
            buf.push(Line::from(Span::styled(line.to_string(), style)));
        }
        buf.push(Line::from(""));
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

    let para = Paragraph::new(buf)
        .style(theme::base())
        .scroll((scroll as u16, 0))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(para, area);
}

fn render_deploy(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let cursor = app.content_cursor;
    let in_content = app.focus == Focus::Content;

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " DEPLOY \u{2014} Select targets and press [d] to run",
        Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let count = app.deploy_targets.len();
    if cursor >= count {
        app.content_cursor = count.saturating_sub(1);
    }

    for (i, dt) in app.deploy_targets.iter().enumerate() {
        let is_cursor = i == app.content_cursor && in_content;
        let check = if dt.selected { "[x]" } else { "[ ]" };
        let label = Pipeline::target_label(&dt.target);

        let prefix = if is_cursor { " >" } else { "  " };
        let fg = if is_cursor { Color::White } else { theme::FG };
        let bg = if is_cursor { theme::BTN_BG } else { theme::BG };
        let mods = if is_cursor { Modifier::BOLD } else { Modifier::empty() };

        lines.push(Line::from(Span::styled(
            format!("{} {} {}", prefix, check, label),
            Style::default().fg(fg).bg(bg).add_modifier(mods),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " [Space] toggle  |  [d] deploy  |  [Tab] switch focus",
        Style::default().fg(theme::COMMENT).add_modifier(Modifier::ITALIC),
    )));

    let border = if in_content {
        theme::SECONDARY
    } else {
        theme::BORDER
    };
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(border));

    let para = Paragraph::new(lines).style(theme::base()).block(block);
    frame.render_widget(para, area);
}

fn render_connect_vps(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let cursor = app.content_cursor;
    let in_content = app.focus == Focus::Content;

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " CONNECT VPS \u{2014} Select a profile and press [Enter]",
        Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let state_icon = match app.connection_state {
        ConnectionState::Connected => "\u{25cf} Connected",
        ConnectionState::Connecting => "\u{25d0} Connecting...",
        ConnectionState::Failed => "\u{2717} Failed",
        ConnectionState::Disconnected => "\u{25cb} Disconnected",
    };
    lines.push(Line::from(Span::styled(
        format!("  Status: {}", state_icon),
        Style::default().fg(theme::COMMENT),
    )));
    lines.push(Line::from(""));

    if app.profiles.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No profiles yet.",
            Style::default().fg(theme::COMMENT),
        )));
        lines.push(Line::from(Span::styled(
            "  Press [a] to add one.",
            Style::default().fg(theme::COMMENT).add_modifier(Modifier::ITALIC),
        )));
    } else {
        let count = app.profiles.len();
        if cursor >= count {
            app.content_cursor = count.saturating_sub(1);
        }

        for (i, p) in app.profiles.iter().enumerate() {
            let is_cursor = i == app.content_cursor && in_content;
            let is_connected = app.selected_profile == Some(i)
                && app.connection_state == ConnectionState::Connected;
            let prefix = if is_cursor { " >" } else { "  " };
            let icon = if is_connected { "\u{25cf}" } else { "\u{25cb}" };
            let fg = if is_cursor { Color::White } else { theme::FG };
            let bg = if is_cursor { theme::BTN_BG } else { theme::BG };
            let mods = if is_cursor { Modifier::BOLD } else { Modifier::empty() };

            lines.push(Line::from(Span::styled(
                format!("{} {} {}@{}:{}", prefix, icon, p.username, p.host, p.port),
                Style::default().fg(fg).bg(bg).add_modifier(mods),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " [Enter] connect  |  [a] add profile  |  [Tab] switch focus",
        Style::default().fg(theme::COMMENT).add_modifier(Modifier::ITALIC),
    )));

    let border = if in_content {
        theme::SECONDARY
    } else {
        theme::BORDER
    };
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(border));

    let para = Paragraph::new(lines).style(theme::base()).block(block);
    frame.render_widget(para, area);
}

fn render_profile_form(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let cursor = app.profile_form.cursor;

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " ADD PROFILE",
        Style::default()
            .fg(theme::YELLOW)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let label_w = 10usize;
    for i in 0..5 {
        let is_active = i == cursor;
        let label = ProfileForm::field_label(i);
        let val = app.profile_form.field_mut(i).map(|s| s.clone()).unwrap_or_default();
        let display = if val.is_empty() {
            match i {
                1 => "required".into(),
                2 => "22".into(),
                4 => "~/.ssh/id_rsa".into(),
                _ => String::new(),
            }
        } else {
            val
        };

        let pad = label_w.saturating_sub(label.len());
        let fg = if is_active { Color::White } else { theme::FG };
        let bg = if is_active { theme::BTN_BG } else { theme::BG };
        let mods = if is_active {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}{}:", " ".repeat(pad), label),
                Style::default().fg(theme::COMMENT),
            ),
            Span::styled(
                format!(" [{}]", display),
                Style::default().fg(fg).bg(bg).add_modifier(mods),
            ),
        ]));
    }

    lines.push(Line::from(""));

    // Save button
    let on_save = cursor == 5;
    let save_fg = if on_save { Color::White } else { theme::BTN_TEXT };
    let save_bg = if on_save { theme::GREEN } else { theme::BTN_BG };
    let save_mods = if on_save {
        Modifier::BOLD
    } else {
        Modifier::empty()
    };
    lines.push(Line::from(Span::styled(
        "   [ Save Profile ]",
        Style::default()
            .fg(save_fg)
            .bg(save_bg)
            .add_modifier(save_mods),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " [Tab/Up/Dn] navigate  |  type to edit  |  [Enter] save",
        Style::default().fg(theme::COMMENT).add_modifier(Modifier::ITALIC),
    )));

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(theme::SECONDARY));

    let para = Paragraph::new(lines).style(theme::base()).block(block);
    frame.render_widget(para, area);
}


