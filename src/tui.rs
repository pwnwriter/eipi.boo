use std::io::Write;
use std::sync::{Arc, Mutex};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};

use crate::canvas;
use crate::confession::{self, BOX_WIDTH, Confession};
use crate::input::InputMode;

#[derive(Clone, Default)]
pub struct TermWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl TermWriter {
    pub fn drain(&self) -> Vec<u8> {
        let mut buf = self.buf.lock().unwrap();
        let data = buf.clone();
        buf.clear();
        data
    }
}

impl Write for TermWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn create_terminal(
    writer: TermWriter,
    width: u16,
    height: u16,
) -> anyhow::Result<Terminal<CrosstermBackend<TermWriter>>> {
    let backend = CrosstermBackend::new(writer);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, width, height)),
        },
    )?;
    Ok(terminal)
}

pub struct RenderState<'a> {
    pub confessions: &'a [Confession],
    pub cam_x: i64,
    pub cam_y: i64,
    pub selected: Option<usize>,
    pub mode: InputMode,
    pub compose_buf: &'a str,
    pub message: Option<&'a str>,
    #[allow(dead_code)]
    pub fingerprint: &'a str,
}

pub fn render(frame: &mut Frame, state: &RenderState) {
    let area = frame.area();
    if area.width < 10 || area.height < 5 {
        let msg = Paragraph::new("Terminal too small").style(Style::default().fg(Color::Red));
        frame.render_widget(msg, area);
        return;
    }

    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    let canvas_area = chunks[0];
    let status_area = chunks[1];

    let visible = canvas::visible_confessions(
        state.confessions,
        state.cam_x,
        state.cam_y,
        canvas_area.width,
        canvas_area.height,
    );

    for &idx in &visible {
        let c = &state.confessions[idx];
        let screen_x = c.x - state.cam_x;
        let screen_y = c.y - state.cam_y;

        if screen_x < 0 || screen_y < 0 {
            continue;
        }

        let sx = screen_x as u16;
        let sy = screen_y as u16;

        if sx >= canvas_area.width || sy >= canvas_area.height {
            continue;
        }

        let box_h = confession::confession_height(&c.text);
        let avail_w = canvas_area.width.saturating_sub(sx).min(BOX_WIDTH);
        let avail_h = canvas_area.height.saturating_sub(sy).min(box_h);

        if avail_w < 6 || avail_h < 3 {
            continue;
        }

        let rect = Rect::new(canvas_area.x + sx, canvas_area.y + sy, avail_w, avail_h);

        let is_selected = state.selected == Some(idx);
        render_confession_box(frame, c, rect, is_selected);
    }

    if state.confessions.is_empty() {
        let hint = Paragraph::new("No confessions yet. Press [n] to write the first one.")
            .style(Style::default().fg(Color::DarkGray));
        let hint_area = Rect::new(
            canvas_area.x + canvas_area.width / 2 - 25,
            canvas_area.y + canvas_area.height / 2,
            50,
            1,
        );
        frame.render_widget(hint, hint_area);
    }

    let status_text = match state.mode {
        InputMode::Browse => {
            if let Some(msg) = state.message {
                msg.to_string()
            } else {
                " [←↑↓→] Scroll  [Tab] Select  [Enter] Upvote  [n] New  [q] Quit".to_string()
            }
        }
        InputMode::Compose => {
            format!(
                " Confess ({}/{}) | [Enter] Submit  [Esc] Cancel",
                state.compose_buf.len(),
                confession::MAX_LENGTH,
            )
        }
    };
    let status =
        Paragraph::new(status_text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(status, status_area);

    if state.mode == InputMode::Compose {
        render_compose(frame, state.compose_buf, area);
    }
}

fn render_confession_box(frame: &mut Frame, c: &Confession, area: Rect, selected: bool) {
    let border_style = if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if c.votes > 50 {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else if c.votes > 10 {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let vote_display = format!("♥ {}", c.votes);

    let block = Block::bordered().border_style(border_style).title_bottom(
        Line::from(Span::styled(vote_display, Style::default().fg(Color::Red))).right_aligned(),
    );

    let text_style = if c.votes > 50 {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else if c.votes > 10 {
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

fn render_compose(frame: &mut Frame, buf: &str, area: Rect) {
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
