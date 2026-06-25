use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::model::confession;

use super::RenderState;

pub fn render(frame: &mut Frame, state: &RenderState, area: Rect) {
    let Some(confession) = state.viewing_confession else {
        return;
    };

    let confession_lines =
        confession::wrap_text(&confession.text, area.width.saturating_sub(4) as usize);
    let confession_h = (confession_lines.len() as u16 + 2).min(area.height / 3);

    let chunks =
        Layout::vertical([Constraint::Length(confession_h), Constraint::Min(0)]).split(area);
    let confession_area = chunks[0];
    let replies_area = chunks[1];

    let heart = if state.voted_ids.contains(&confession.id) {
        "󰋑"
    } else {
        "♥"
    };
    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Yellow))
        .title_bottom(
            Line::from(Span::styled(
                format!("{} {}", heart, confession.votes),
                Style::default().fg(Color::Red),
            ))
            .right_aligned(),
        );

    let p = Paragraph::new(confession.text.as_str())
        .block(block)
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(p, confession_area);

    if state.replies.is_empty() {
        if replies_area.height > 0 {
            let hint = Paragraph::new("  No replies yet. Press [r] to reply.")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(
                hint,
                Rect::new(replies_area.x, replies_area.y, replies_area.width, 1),
            );
        }
    } else {
        let mut y = replies_area.y;
        let visible_replies = state.replies.iter().skip(state.reply_scroll);

        for reply in visible_replies {
            if y >= replies_area.y + replies_area.height {
                break;
            }

            let age = confession::time_ago(&reply.replied_at);
            let name_line = Line::from(vec![
                Span::styled("  ↳ ", Style::default().fg(Color::DarkGray)),
                Span::styled(&reply.name, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("  {}", age),
                    Style::default().fg(Color::Indexed(242)),
                ),
            ]);
            frame.render_widget(
                Paragraph::new(name_line),
                Rect::new(replies_area.x, y, replies_area.width, 1),
            );
            y += 1;

            let wrapped =
                confession::wrap_text(&reply.text, replies_area.width.saturating_sub(6) as usize);
            for line in &wrapped {
                if y >= replies_area.y + replies_area.height {
                    break;
                }
                let text_line = Line::from(vec![
                    Span::raw("      "),
                    Span::styled(line.as_str(), Style::default().fg(Color::Gray)),
                ]);
                frame.render_widget(
                    Paragraph::new(text_line),
                    Rect::new(replies_area.x, y, replies_area.width, 1),
                );
                y += 1;
            }

            if y < replies_area.y + replies_area.height {
                y += 1;
            }
        }
    }
}
