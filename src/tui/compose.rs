use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

pub fn render_confession(frame: &mut Frame, buf: &str, area: Rect) {
    let popup_w = 50u16.min(area.width.saturating_sub(4));
    let popup_h = 8u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = (area.height.saturating_sub(popup_h)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

    frame.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Yellow))
        .title(" New Confession ");

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let display_text = if buf.is_empty() {
        "Type your confession...".to_string()
    } else {
        format!("{}_", buf)
    };

    let text_style = if buf.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let paragraph = Paragraph::new(display_text)
        .style(text_style)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, inner);
}

pub fn render_reply(frame: &mut Frame, buf: &str, name: &str, area: Rect) {
    let popup_w = 50u16.min(area.width.saturating_sub(4));
    let popup_h = 8u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = (area.height.saturating_sub(popup_h)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

    frame.render_widget(Clear, popup_area);

    let title = if name.is_empty() {
        " Reply as anon ".to_string()
    } else {
        format!(" Reply as {} ", name)
    };

    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Cyan))
        .title(title);

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let display_text = if buf.is_empty() {
        "Type your reply...".to_string()
    } else {
        format!("{}_", buf)
    };

    let text_style = if buf.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let paragraph = Paragraph::new(display_text)
        .style(text_style)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, inner);
}

pub fn render_quit(frame: &mut Frame, area: Rect) {
    let popup_w = 40u16.min(area.width.saturating_sub(4));
    let popup_h = 7u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = (area.height.saturating_sub(popup_h)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

    frame.render_widget(Clear, popup_area);

    let block = Block::bordered().border_style(Style::default().fg(Color::Red));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

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

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}
