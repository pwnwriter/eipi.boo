use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::helper::consts;
use crate::model::confession::{self, Confession};

pub fn render(frame: &mut Frame, c: &Confession, area: Rect, selected: bool, has_voted: bool) {
    let border_style = if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if c.votes > consts::VOTES_MAGENTA {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else if c.votes > consts::VOTES_CYAN {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let heart = if has_voted { "󰋑" } else { "♥" };
    let reply_str = if c.reply_count > 0 {
        format!("󰍧 {}  ", c.reply_count)
    } else {
        String::new()
    };
    let vote_display = format!("{}{} {}", reply_str, heart, c.votes);

    let age = confession::time_ago(&c.created_at);

    let mut block = Block::bordered()
        .border_style(border_style)
        .title_top(
            Line::from(Span::styled(
                format!(" {} ", age),
                Style::default().fg(Color::Indexed(242)),
            ))
            .right_aligned(),
        )
        .title_bottom(
            Line::from(Span::styled(vote_display, Style::default().fg(Color::Red))).right_aligned(),
        );

    if selected {
        block = block.title(Line::from(Span::styled(
            " ▶ ",
            Style::default().fg(Color::Yellow),
        )));
    }

    let text_style = if c.votes > consts::VOTES_MAGENTA {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else if c.votes > consts::VOTES_CYAN {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    let paragraph = Paragraph::new(c.text.as_str())
        .block(block)
        .style(text_style)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
