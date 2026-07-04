use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::theme::Theme;

fn centered_popup(frame: &mut Frame, area: Rect, w: u16, h: u16) -> Rect {
    let max_w = (area.width * 9 / 10).max(20);
    let pw = w.min(max_w).min(area.width.saturating_sub(2));
    let ph = h.min(area.height.saturating_sub(2));
    let popup = Rect::new(
        (area.width.saturating_sub(pw)) / 2,
        (area.height.saturating_sub(ph)) / 2,
        pw,
        ph,
    );
    frame.render_widget(Clear, popup);
    popup
}

fn text_input(frame: &mut Frame, buf: &str, placeholder: &str, inner: Rect, theme: &Theme) {
    let (text, style) = if buf.is_empty() {
        (placeholder.to_string(), Style::default().fg(theme.text_dim))
    } else {
        (format!("{}_", buf), Style::default().fg(theme.text))
    };
    frame.render_widget(
        Paragraph::new(text).style(style).wrap(Wrap { trim: true }),
        inner,
    );
}

pub fn render_confession(frame: &mut Frame, buf: &str, area: Rect, theme: &Theme) {
    let popup = centered_popup(frame, area, 50, 8);
    let block = Block::bordered()
        .border_style(Style::default().fg(theme.accent))
        .title(" New Confession ");
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    text_input(frame, buf, "Type your confession...", inner, theme);
}

pub fn render_reply(frame: &mut Frame, buf: &str, name: &str, area: Rect, theme: &Theme) {
    let popup = centered_popup(frame, area, 50, 8);
    let title = if name.is_empty() {
        " Reply as anon ".to_string()
    } else {
        format!(" Reply as {} ", name)
    };
    let block = Block::bordered()
        .border_style(Style::default().fg(theme.accent_alt))
        .title(title);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    text_input(frame, buf, "Type your reply...", inner, theme);
}

pub fn render_search(frame: &mut Frame, buf: &str, area: Rect, theme: &Theme) {
    let popup = centered_popup(frame, area, 50, 5);
    let block = Block::bordered()
        .border_style(Style::default().fg(theme.accent_search))
        .title(" Search ");
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    text_input(frame, buf, "Type to search confessions...", inner, theme);
}

pub fn render_quit(frame: &mut Frame, area: Rect, theme: &Theme, created_confession: bool) {
    let popup = centered_popup(frame, area, 40, 7);
    let block = Block::bordered().border_style(Style::default().fg(theme.heart));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "wait, leaving already? :(",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
    ];
    if !created_confession {
        lines.push(Line::from(Span::styled(
            "did you confess something?",
            Style::default().fg(theme.text_dim),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("q/⏎ ", Style::default().fg(theme.heart)),
        Span::styled("leave   ", Style::default().fg(theme.text_dim)),
        Span::styled("any key ", Style::default().fg(theme.online)),
        Span::styled("stay", Style::default().fg(theme.text_dim)),
    ]));

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}
