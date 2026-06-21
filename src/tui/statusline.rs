use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::confession;
use crate::input::InputMode;

use super::RenderState;

const DIM: Color = Color::DarkGray;
const KEY: Color = Color::White;

fn hint<'a>(key: &'a str, label: &'a str) -> Vec<Span<'a>> {
    vec![
        Span::styled(key, Style::default().fg(KEY)),
        Span::styled(format!(" {}   ", label), Style::default().fg(DIM)),
    ]
}

pub fn render(frame: &mut Frame, state: &RenderState, area: Rect) {
    if area.height < 3 {
        return;
    }

    let info_area = Rect::new(area.x, area.y, area.width, 1);
    let rule_area = Rect::new(area.x, area.y + 1, area.width, 1);
    let hints_area = Rect::new(area.x, area.y + 2, area.width, 1);

    let info_line = match state.mode {
        InputMode::Browse => Line::from(vec![
            Span::styled(
                format!("{} confessions", state.total_confessions),
                Style::default().fg(Color::Indexed(242)),
            ),
            Span::styled(" · ", Style::default().fg(Color::Indexed(238))),
            Span::styled(
                format!("{} humans", state.total_humans),
                Style::default().fg(Color::Indexed(242)),
            ),
            Span::styled(" · ", Style::default().fg(Color::Indexed(238))),
            Span::styled(
                "pwnwriter/eipi.boo",
                Style::default().fg(Color::Indexed(242)),
            ),
        ])
        .centered(),
        _ => Line::from(""),
    };
    let info_p = Paragraph::new(info_line).style(Style::default().fg(Color::Indexed(238)));
    frame.render_widget(info_p, info_area);

    let rule = "─".repeat(area.width as usize);
    let rule_p = Paragraph::new(rule).style(Style::default().fg(Color::Indexed(238)));
    frame.render_widget(rule_p, rule_area);

    if let Some(msg) = state.message
        && matches!(state.mode, InputMode::Browse | InputMode::ViewReplies)
    {
        let line = Line::from(msg).centered();
        let p = Paragraph::new(line).style(Style::default().fg(Color::Yellow));
        frame.render_widget(p, hints_area);
        return;
    }

    let mut spans: Vec<Span> = Vec::new();

    match state.mode {
        InputMode::Browse => {
            spans.extend(hint("↑↓/jk", "scroll"));
            spans.extend(hint("tab", "select"));
            spans.extend(hint("v", "vote"));
            spans.extend(hint("⏎", "replies"));
            spans.extend(hint("n", "confess"));
            spans.extend(hint("q", "quit"));
        }
        InputMode::Compose => {
            spans.push(Span::styled(
                format!("{}/{}", state.compose_buf.len(), confession::MAX_LENGTH),
                Style::default().fg(DIM),
            ));
            spans.push(Span::raw("   "));
            spans.extend(hint("⏎", "submit"));
            spans.extend(hint("esc", "cancel"));
        }
        InputMode::ViewReplies => {
            spans.push(Span::styled(
                format!("{} replies", state.replies.len()),
                Style::default().fg(DIM),
            ));
            spans.push(Span::raw("   "));
            spans.extend(hint("r", "reply"));
            spans.extend(hint("↑↓/jk", "scroll"));
            spans.extend(hint("v", "vote"));
            spans.extend(hint("esc", "back"));
        }
        InputMode::ComposeReply => {
            if state.reply_name_phase {
                spans.push(Span::styled("name (optional): ", Style::default().fg(DIM)));
                spans.push(Span::styled(
                    format!("{}_", state.reply_name_buf),
                    Style::default().fg(KEY),
                ));
                spans.push(Span::raw("   "));
                spans.extend(hint("⏎", "next"));
                spans.extend(hint("esc", "cancel"));
            } else {
                spans.push(Span::styled(
                    format!("{}/{}", state.compose_buf.len(), crate::reply::MAX_LENGTH),
                    Style::default().fg(DIM),
                ));
                spans.push(Span::raw("   "));
                spans.extend(hint("⏎", "submit"));
                spans.extend(hint("esc", "cancel"));
            }
        }
        InputMode::ConfirmQuit => {}
    }

    let line = Line::from(spans).centered();
    frame.render_widget(Paragraph::new(line), hints_area);
}
