use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::helper::consts;
use crate::model::confession;

use super::RenderState;

pub fn render(frame: &mut Frame, state: &RenderState, area: Rect) {
    if state.confessions.is_empty() {
        let hint = Paragraph::new("No confessions yet. Press [n] to write one.")
            .style(Style::default().fg(Color::DarkGray));
        let cx = area.x + area.width.saturating_sub(46) / 2;
        let cy = area.y + area.height / 2;
        frame.render_widget(hint, Rect::new(cx, cy, 46.min(area.width), 1));
        return;
    }

    let idx = state.card_index.min(state.confessions.len() - 1);
    let c = &state.confessions[idx];
    let has_voted = state.voted_ids.contains(&c.id);

    // responsive card width: use 60% of area, clamped to min/max
    let desired = (area.width * 3 / 5).clamp(consts::CARD_MIN_W, consts::CARD_MAX_W);
    let card_w = desired.min(area.width.saturating_sub(2)) as usize;
    if card_w < consts::CARD_MIN_W as usize {
        return;
    }
    let iw = card_w - 2; // inner width between │ borders

    let wrapped = confession::wrap_text(&c.text, iw.saturating_sub(4));
    let text_lines = wrapped.len() as u16;

    // card: top(1) + dots(1) + sep(1) + pad(1) + text + pad(1) + sep(1) + footer(1) + bottom(1) = 8 + text
    let card_h = 8 + text_lines;

    // check if character fits below
    let show_char = area.height >= card_h + 4 + 2; // +2 for centering margin
    let total_h = if show_char { card_h + 4 } else { card_h };

    if total_h > area.height {
        return;
    }

    let cx = area.x + area.width.saturating_sub(card_w as u16) / 2;
    let cy = area.y + area.height.saturating_sub(total_h) / 2;

    let border = Style::default().fg(Color::Indexed(242));
    let dim = Style::default().fg(Color::Indexed(238));
    let text_style = Style::default().fg(Color::White);
    let age_style = Style::default().fg(Color::Indexed(242));
    let char_style = Style::default().fg(Color::Indexed(242));

    let age = confession::time_ago(&c.created_at);
    let heart = if has_voted { "♥" } else { "♡" };
    let heart_style = if has_voted {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let position = format!("{}/{}", idx + 1, state.confessions.len());

    let mut lines: Vec<Line> = Vec::new();

    // ╭────╮
    lines.push(Line::from(Span::styled(
        format!("╭{}╮", "─".repeat(iw)),
        border,
    )));

    // │  ● ● ●              age │
    let age_display = format!(" {} ", age);
    let age_dcols = age.len() + 2;
    let dots_dcols = 7; // "  ● ● ●" = 7 display cols
    let dots_pad = iw.saturating_sub(dots_dcols + age_dcols);
    lines.push(Line::from(vec![
        Span::styled("│", border),
        Span::styled("  ", dim),
        Span::styled("●", Style::default().fg(Color::Red)),
        Span::styled(" ●", Style::default().fg(Color::Yellow)),
        Span::styled(" ●", Style::default().fg(Color::Green)),
        Span::raw(" ".repeat(dots_pad)),
        Span::styled(age_display, age_style),
        Span::styled("│", border),
    ]));

    // │──────│
    lines.push(Line::from(Span::styled(
        format!("│{}│", "─".repeat(iw)),
        border,
    )));

    // │      │
    lines.push(Line::from(vec![
        Span::styled("│", border),
        Span::raw(" ".repeat(iw)),
        Span::styled("│", border),
    ]));

    // text lines
    for line in &wrapped {
        let dcols = line.chars().count();
        let right_pad = iw.saturating_sub(dcols + 2);
        lines.push(Line::from(vec![
            Span::styled("│", border),
            Span::raw("  "),
            Span::styled(line.as_str(), text_style),
            Span::raw(" ".repeat(right_pad)),
            Span::styled("│", border),
        ]));
    }

    // │      │
    lines.push(Line::from(vec![
        Span::styled("│", border),
        Span::raw(" ".repeat(iw)),
        Span::styled("│", border),
    ]));

    // │──────│
    lines.push(Line::from(Span::styled(
        format!("│{}│", "─".repeat(iw)),
        border,
    )));

    // footer: │  ♡ 3  󰍧 2        3/12  │
    let votes_str = format!(" {}", c.votes);
    let votes_dcols = votes_str.len();
    let heart_dcols: usize = 1;

    let (reply_display, reply_dcols) = if c.reply_count > 0 {
        let s = format!("  \u{F0367} {}", c.reply_count);
        let dcols = 2 + 1 + 1 + c.reply_count.to_string().len();
        (s, dcols)
    } else {
        (String::new(), 0)
    };

    let pos_dcols = position.len();
    let left_dcols = 2 + heart_dcols + votes_dcols + reply_dcols;
    let right_dcols = pos_dcols + 2;
    let footer_pad = iw.saturating_sub(left_dcols + right_dcols);

    lines.push(Line::from(vec![
        Span::styled("│", border),
        Span::raw("  "),
        Span::styled(heart, heart_style),
        Span::styled(votes_str, heart_style),
        Span::styled(reply_display, Style::default().fg(Color::Cyan)),
        Span::raw(" ".repeat(footer_pad)),
        Span::styled(&position, age_style),
        Span::raw("  "),
        Span::styled("│", border),
    ]));

    // bottom border
    if show_char {
        // ╰──────┬──────╯
        let mid = iw / 2;
        lines.push(Line::from(Span::styled(
            format!("╰{}┬{}╯", "─".repeat(mid), "─".repeat(iw - mid - 1)),
            border,
        )));

        // character holding the card — face changes with confession length
        let center = mid + 1;
        let face = match c.text.len() {
            0..70 => "\\(^_^)/",
            70..150 => "\\(o_o)/",
            150..220 => "\\(>_<)/",
            _ => "\\(x_x)/",
        };
        lines.push(Line::from(Span::styled(
            format!("{}│", " ".repeat(center)),
            char_style,
        )));
        lines.push(Line::from(Span::styled(
            format!("{}{}", " ".repeat(center.saturating_sub(3)), face),
            char_style,
        )));
        lines.push(Line::from(Span::styled(
            format!("{}│", " ".repeat(center)),
            char_style,
        )));
        lines.push(Line::from(Span::styled(
            format!("{}/ \\", " ".repeat(center.saturating_sub(1))),
            char_style,
        )));
    } else {
        // ╰──────╯
        lines.push(Line::from(Span::styled(
            format!("╰{}╯", "─".repeat(iw)),
            border,
        )));
    }

    let paragraph = Paragraph::new(lines);
    let card_rect = Rect::new(cx, cy, card_w as u16, total_h);
    frame.render_widget(paragraph, card_rect);

    if c.votes > consts::VOTES_GLOW {
        super::glow::render_ring(frame, c.votes, card_rect, area);
    }
}
