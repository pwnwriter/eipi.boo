mod card_view;
mod compose;
mod confession_box;
mod glow;
mod reply_panel;
mod statusline;

use std::io::Write;
use std::sync::{Arc, Mutex};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};

use crate::canvas;
use crate::confession::{self, BOX_WIDTH, Confession};
use crate::input::InputMode;
use crate::reply::Reply;

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
    pub reply_name_buf: &'a str,
    pub reply_name_phase: bool,
    pub message: Option<&'a str>,
    pub total_confessions: i64,
    pub total_humans: i64,
    pub voted_ids: &'a [i64],
    pub replies: &'a [Reply],
    pub viewing_confession: Option<&'a Confession>,
    pub reply_scroll: usize,
    pub card_index: usize,
    pub came_from_card: bool,
}

pub fn render(frame: &mut Frame, state: &RenderState) {
    let area = frame.area();
    if area.width < 10 || area.height < 5 {
        let msg = Paragraph::new("Terminal too small").style(Style::default().fg(Color::Red));
        frame.render_widget(msg, area);
        return;
    }

    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(area);
    let main_area = chunks[0];
    let status_area = chunks[1];

    let card_reply = state.came_from_card
        && matches!(state.mode, InputMode::ViewReplies | InputMode::ComposeReply);

    if state.mode == InputMode::CardView {
        card_view::render(frame, state, main_area);
        statusline::render(frame, state, status_area);
        return;
    }

    if card_reply {
        let half = main_area.width / 2;
        let h_chunks =
            Layout::horizontal([Constraint::Length(half), Constraint::Min(0)]).split(main_area);
        card_view::render(frame, state, h_chunks[0]);
        reply_panel::render(frame, state, h_chunks[1]);
        statusline::render(frame, state, status_area);

        if state.mode == InputMode::ComposeReply && !state.reply_name_phase {
            compose::render_reply(frame, state.compose_buf, state.reply_name_buf, area);
        }
        return;
    }

    let reply_open = matches!(state.mode, InputMode::ViewReplies | InputMode::ComposeReply);

    let (canvas_area, reply_area) = if reply_open {
        let half = main_area.width / 2;
        let h_chunks =
            Layout::horizontal([Constraint::Length(half), Constraint::Min(0)]).split(main_area);
        (h_chunks[0], Some(h_chunks[1]))
    } else {
        (main_area, None)
    };

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
        let has_voted = state.voted_ids.contains(&c.id);
        confession_box::render(frame, c, rect, is_selected, has_voted);
    }

    glow::render(frame, state.confessions, state.cam_x, state.cam_y, canvas_area);

    if state.confessions.is_empty() {
        let hint = Paragraph::new("No confessions yet. Press [n] to write the first one.")
            .style(Style::default().fg(Color::DarkGray));
        let cx = canvas_area.x + canvas_area.width.saturating_sub(50) / 2;
        let cy = canvas_area.y + canvas_area.height / 2;
        let hw = 50.min(canvas_area.width);
        frame.render_widget(hint, Rect::new(cx, cy, hw, 1));
    }

    if let Some(rarea) = reply_area {
        reply_panel::render(frame, state, rarea);
    }

    statusline::render(frame, state, status_area);

    if state.mode == InputMode::Compose {
        compose::render_confession(frame, state.compose_buf, area);
    }

    if state.mode == InputMode::ComposeReply && !state.reply_name_phase {
        compose::render_reply(frame, state.compose_buf, state.reply_name_buf, area);
    }

    if state.mode == InputMode::ConfirmQuit {
        compose::render_quit(frame, area);
    }
}
