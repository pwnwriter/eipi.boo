use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

fn centered_popup(frame: &mut Frame, area: Rect, w: u16, h: u16) -> Rect {
    let pw = w.min(area.width.saturating_sub(4));
    let ph = h.min(area.height.saturating_sub(4));
    let popup = Rect::new(
        (area.width.saturating_sub(pw)) / 2,
        (area.height.saturating_sub(ph)) / 2,
        pw,
        ph,
    );
    frame.render_widget(Clear, popup);
    popup
}

fn text_input(frame: &mut Frame, buf: &str, placeholder: &str, inner: Rect) {
    let (text, style) = if buf.is_empty() {
        (placeholder.to_string(), Style::default().fg(Color::DarkGray))
    } else {
        (format!("{}_", buf), Style::default().fg(Color::White))
    };
    frame.render_widget(
        Paragraph::new(text).style(style).wrap(Wrap { trim: true }),
        inner,
    );
}

pub fn render_confession(frame: &mut Frame, buf: &str, area: Rect) {
    let popup = centered_popup(frame, area, 50, 8);
    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Yellow))
        .title(" New Confession ");
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    text_input(frame, buf, "Type your confession...", inner);
}

pub fn render_reply(frame: &mut Frame, buf: &str, name: &str, area: Rect) {
    let popup = centered_popup(frame, area, 50, 8);
    let title = if name.is_empty() {
        " Reply as anon ".to_string()
    } else {
        format!(" Reply as {} ", name)
    };
    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Cyan))
        .title(title);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    text_input(frame, buf, "Type your reply...", inner);
}

pub fn render_quit(frame: &mut Frame, area: Rect) {
    let popup = centered_popup(frame, area, 40, 7);
    let block = Block::bordered().border_style(Style::default().fg(Color::Red));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "wait, leaving already? :(",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "did you confess something?",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("q/⏎ ", Style::default().fg(Color::Red)),
            Span::styled("leave   ", Style::default().fg(Color::DarkGray)),
            Span::styled("any key ", Style::default().fg(Color::Green)),
            Span::styled("stay", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}
