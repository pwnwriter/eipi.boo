use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use super::keybinds::HELP_KEYBINDS;
use super::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
    // header chrome (7 rows) + keybinds + trailing rule + link
    let n = HELP_KEYBINDS.len();
    let total_rows = 9 + n;
    let popup = centered_popup(frame, area, 60, total_rows as u16 + 2);
    let block = Block::bordered().border_style(Style::default().fg(theme.border));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let rows = Layout::vertical(vec![Constraint::Length(1); total_rows]).split(inner);

    render_line(
        frame,
        rows[1],
        Line::from(Span::styled(
            "eipi.boo",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
        .centered(),
    );
    render_line(
        frame,
        rows[2],
        Line::from(Span::styled(
            "Confessions over ssh",
            Style::default().fg(theme.accent_search),
        ))
        .centered(),
    );
    render_line(
        frame,
        rows[3],
        Line::from(Span::styled(
            "browse, reply, search, and disappear.",
            Style::default().fg(theme.text_dim),
        ))
        .centered(),
    );
    render_rule(frame, rows[4], theme);
    render_header(frame, rows[5], theme);

    for (i, hint) in HELP_KEYBINDS.iter().enumerate() {
        render_keybind_row(frame, rows[7 + i], hint.key, hint.label, theme);
    }

    render_rule(frame, rows[7 + n], theme);
    render_line(
        frame,
        rows[8 + n],
        Line::from(Span::styled(
            " github.com/pwnwriter/eipi.boo/issues/new",
            Style::default().fg(theme.text_dim),
        )),
    );
}

fn centered_popup(frame: &mut Frame, area: Rect, w: u16, h: u16) -> Rect {
    let pw = w.min(area.width.saturating_sub(2));
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

fn render_line(frame: &mut Frame, area: Rect, line: Line<'static>) {
    frame.render_widget(Paragraph::new(line), area);
}

fn render_rule(frame: &mut Frame, area: Rect, theme: &Theme) {
    frame.render_widget(
        Paragraph::new("─".repeat(area.width as usize))
            .style(Style::default().fg(theme.border_dim)),
        area,
    );
}

fn render_header(frame: &mut Frame, area: Rect, theme: &Theme) {
    let cols = Layout::horizontal([Constraint::Min(0), Constraint::Length(18)]).split(area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " keybinds",
            Style::default().fg(theme.accent),
        ))),
        cols[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("close: ", Style::default().fg(theme.text_dim)),
            Span::styled("esc q ? enter", Style::default().fg(theme.warning)),
        ])),
        cols[1],
    );
}

fn render_keybind_row(frame: &mut Frame, area: Rect, key: &str, value: &str, theme: &Theme) {
    let cols = Layout::horizontal([
        Constraint::Length(18),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {}", key),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))),
        cols[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "│ ",
            Style::default().fg(theme.border_dim),
        ))),
        cols[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            value.to_string(),
            Style::default().fg(theme.text_secondary),
        ))),
        cols[2],
    );
}
