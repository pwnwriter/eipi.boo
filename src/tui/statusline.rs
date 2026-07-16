use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::model::confession;
use crate::server::input::InputMode;

use super::RenderState;
use super::keybinds::{KeybindHint, status_hints};

pub fn render(frame: &mut Frame, state: &RenderState, area: Rect) {
    if area.height < 3 {
        return;
    }

    let theme = &state.theme;

    let info_area = Rect::new(area.x, area.y, area.width, 1);
    let rule_area = Rect::new(area.x, area.y + 1, area.width, 1);
    let hints_area = Rect::new(area.x, area.y + 2, area.width, 1);

    let info_line = match state.mode {
        InputMode::Browse | InputMode::CardView | InputMode::SearchResults => Line::from(vec![
            Span::styled(
                format!("{} confessions", state.total_confessions),
                Style::default().fg(theme.text_dim),
            ),
            Span::styled(" · ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{} humans", state.total_humans),
                Style::default().fg(theme.text_dim),
            ),
            Span::styled(" · ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{} online", state.online),
                Style::default().fg(theme.online),
            ),
            Span::styled(" · ", Style::default().fg(theme.text_dim)),
            Span::styled("pwnwriter/eipi.boo", Style::default().fg(theme.text_dim)),
        ])
        .centered(),
        _ => Line::from(""),
    };
    let info_p = Paragraph::new(info_line).style(Style::default().fg(theme.text_dim));
    frame.render_widget(info_p, info_area);

    let rule = "─".repeat(area.width as usize);
    let rule_p = Paragraph::new(rule).style(Style::default().fg(theme.text_dim));
    frame.render_widget(rule_p, rule_area);

    if let Some(msg) = state.message
        && matches!(
            state.mode,
            InputMode::Browse | InputMode::CardView | InputMode::ViewReplies
        )
    {
        let line = Line::from(msg).centered();
        let p = Paragraph::new(line).style(Style::default().fg(theme.warning));
        frame.render_widget(p, hints_area);
        return;
    }

    if let Some(link) = state.share_link
        && matches!(
            state.mode,
            InputMode::Browse | InputMode::CardView | InputMode::ViewReplies
        )
    {
        let line = Line::from(vec![
            Span::styled("share ", Style::default().fg(theme.text_dim)),
            Span::styled(link, Style::default().fg(theme.accent_search)),
            Span::styled("  ·  s to hide", Style::default().fg(theme.text_dim)),
        ])
        .centered();
        frame.render_widget(Paragraph::new(line), hints_area);
        return;
    }

    let mut leading: Vec<Span> = Vec::new();

    match state.mode {
        InputMode::Compose => {
            leading.push(Span::styled(
                format!("{}/{}", state.compose_buf.len(), confession::MAX_LENGTH),
                Style::default().fg(theme.text_dim),
            ));
            leading.push(Span::raw("   "));
        }
        InputMode::ViewReplies => {
            leading.push(Span::styled(
                format!("{} replies", state.replies.len()),
                Style::default().fg(theme.text_dim),
            ));
            leading.push(Span::raw("   "));
        }
        InputMode::ComposeReply => {
            if state.reply_name_phase {
                leading.push(Span::styled(
                    "name (optional): ",
                    Style::default().fg(theme.text_dim),
                ));
                leading.push(Span::styled(
                    format!("{}_", state.reply_name_buf),
                    Style::default().fg(theme.text),
                ));
                leading.push(Span::raw("   "));
            } else {
                leading.push(Span::styled(
                    format!(
                        "{}/{}",
                        state.compose_buf.len(),
                        crate::model::reply::MAX_LENGTH
                    ),
                    Style::default().fg(theme.text_dim),
                ));
                leading.push(Span::raw("   "));
            }
        }
        InputMode::SearchResults => {
            leading.push(Span::styled(
                format!(
                    "{}/{} matches",
                    state.search_index + 1,
                    state.search_result_count
                ),
                Style::default().fg(theme.accent_search),
            ));
            leading.push(Span::raw("   "));
        }
        InputMode::Browse
        | InputMode::CardView
        | InputMode::ReactionPicker
        | InputMode::Search
        | InputMode::ConfirmQuit
        | InputMode::Splash
        | InputMode::ThemePicker
        | InputMode::Help => {}
    }

    render_hints_row(
        frame,
        hints_area,
        &leading,
        status_hints(state.mode, state.reply_name_phase),
    );
}

fn render_hints_row(frame: &mut Frame, area: Rect, leading: &[Span<'_>], hints: &[KeybindHint]) {
    let segments = hint_segments(hints);
    let left_width = if leading.is_empty() {
        0
    } else {
        line_width(leading) as u16
    };
    let gap_width = if leading.is_empty() { 0 } else { 1 };
    let right_width = segments.iter().map(String::len).sum::<usize>() as u16;
    let used = left_width
        .saturating_add(gap_width)
        .saturating_add(right_width);
    let remaining = area.width.saturating_sub(used);
    let left_pad = remaining / 2;
    let right_pad = remaining - left_pad;

    let chunks = Layout::horizontal([
        Constraint::Length(left_pad),
        Constraint::Length(left_width),
        Constraint::Length(gap_width),
        Constraint::Length(right_width),
        Constraint::Length(right_pad),
    ])
    .split(area);

    if !leading.is_empty() {
        frame.render_widget(Paragraph::new(Line::from(leading.to_vec())), chunks[1]);
    }

    let hint_chunks = Layout::horizontal(
        segments
            .iter()
            .map(|segment| Constraint::Length(segment.len() as u16))
            .collect::<Vec<_>>(),
    )
    .split(chunks[3]);

    for (chunk, hint) in hint_chunks.iter().zip(hints.iter().copied()) {
        frame.render_widget(Paragraph::new(Line::from(render_hint(hint))), *chunk);
    }
}

fn render_hint(hint: KeybindHint) -> Vec<Span<'static>> {
    vec![
        Span::styled(hint.key, Style::default().fg(Color::White)),
        Span::styled(
            format!(" {}", hint.label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("   "),
    ]
}

fn hint_segments(hints: &[KeybindHint]) -> Vec<String> {
    hints
        .iter()
        .map(|hint| format!("{} {}   ", hint.key, hint.label))
        .collect()
}

fn line_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|span| span.content.len()).sum()
}
