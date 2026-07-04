pub(crate) mod canvas;
mod card_view;
mod compose;
mod confession_box;
mod help;
mod keybinds;
mod reaction_picker;
mod reactions;
mod reply_panel;
pub(crate) mod splash;
mod statusline;
pub(crate) mod theme;
pub(crate) mod theme_picker;
pub(crate) mod themes;

use std::io::Write;
use std::sync::Arc;

use parking_lot::Mutex;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};

use crate::model::confession::{self, BOX_WIDTH, Confession};
use crate::model::reply::Reply;
use crate::server::input::InputMode;

#[derive(Clone, Default)]
pub struct TermWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl TermWriter {
    pub fn drain(&self) -> Vec<u8> {
        let mut buf = self.buf.lock();
        let data = buf.clone();
        buf.clear();
        data
    }
}

impl Write for TermWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().extend_from_slice(data);
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
    pub reaction_return_mode: InputMode,
    pub compose_buf: &'a str,
    pub reply_name_buf: &'a str,
    pub reply_name_phase: bool,
    pub message: Option<&'a str>,
    pub total_confessions: i64,
    pub total_humans: i64,
    pub online: usize,
    pub replies: &'a [Reply],
    pub viewing_confession: Option<&'a Confession>,
    pub reply_scroll: usize,
    pub card_index: usize,
    pub came_from_card: bool,
    pub created_confession: bool,
    pub search_buf: &'a str,
    pub search_result_count: usize,
    pub search_index: usize,
    pub splash_frame: u8,
    pub theme: theme::Theme,
    pub theme_picker_index: usize,
    pub reaction_picker_index: usize,
    pub render_tick: u64,
}

pub fn render(frame: &mut Frame, state: &RenderState) {
    let area = frame.area();
    if area.width < 10 || area.height < 5 {
        let msg = Paragraph::new("Terminal too small").style(Style::default().fg(Color::Red));
        frame.render_widget(msg, area);
        return;
    }

    let theme = &state.theme;

    if state.mode == InputMode::Splash {
        splash::render(frame, state.splash_frame, area, theme);
        return;
    }

    let effective_mode = if state.mode == InputMode::ReactionPicker {
        state.reaction_return_mode
    } else {
        state.mode
    };

    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(area);
    let main_area = chunks[0];
    let status_area = chunks[1];

    let card_reply = state.came_from_card
        && matches!(
            effective_mode,
            InputMode::ViewReplies | InputMode::ComposeReply
        );

    if effective_mode == InputMode::CardView || effective_mode == InputMode::SearchResults {
        card_view::render(frame, state, main_area);
        statusline::render(frame, state, status_area);
        if state.mode == InputMode::ReactionPicker
            && let Some(card_rect) = card_view::card_rect(state, main_area)
        {
            reaction_picker::render(frame, card_rect, state.reaction_picker_index, area, theme);
        }
        return;
    }

    if card_reply {
        let half = main_area.width / 2;
        let h_chunks =
            Layout::horizontal([Constraint::Length(half), Constraint::Min(0)]).split(main_area);
        card_view::render(frame, state, h_chunks[0]);
        reply_panel::render(frame, state, h_chunks[1]);
        statusline::render(frame, state, status_area);

        if effective_mode == InputMode::ComposeReply && !state.reply_name_phase {
            compose::render_reply(frame, state.compose_buf, state.reply_name_buf, area, theme);
        }
        if state.mode == InputMode::ReactionPicker
            && let Some(card_rect) = card_view::card_rect(state, h_chunks[0])
        {
            reaction_picker::render(frame, card_rect, state.reaction_picker_index, area, theme);
        }
        return;
    }

    let reply_open = matches!(
        effective_mode,
        InputMode::ViewReplies | InputMode::ComposeReply
    );

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
        confession_box::render(frame, c, rect, is_selected, theme);
        reactions::render(frame, c, rect, state.render_tick, theme);
    }

    if state.confessions.is_empty() {
        let hint = Paragraph::new("No confessions yet. Press [n] to write the first one.")
            .style(Style::default().fg(theme.text_dim));
        let cx = canvas_area.x + canvas_area.width.saturating_sub(50) / 2;
        let cy = canvas_area.y + canvas_area.height / 2;
        let hw = 50.min(canvas_area.width);
        frame.render_widget(hint, Rect::new(cx, cy, hw, 1));
    }

    if let Some(rarea) = reply_area {
        reply_panel::render(frame, state, rarea);
    }

    statusline::render(frame, state, status_area);

    if effective_mode == InputMode::Compose {
        compose::render_confession(frame, state.compose_buf, area, theme);
    }

    if effective_mode == InputMode::ComposeReply && !state.reply_name_phase {
        compose::render_reply(frame, state.compose_buf, state.reply_name_buf, area, theme);
    }

    if effective_mode == InputMode::Search {
        compose::render_search(frame, state.search_buf, area, theme);
    }

    if effective_mode == InputMode::ConfirmQuit {
        compose::render_quit(frame, area, theme, state.created_confession);
    }

    if effective_mode == InputMode::ThemePicker {
        theme_picker::render(frame, state.theme_picker_index, theme, area);
    }

    if state.mode == InputMode::Help {
        help::render(frame, area, theme);
    }

    if state.mode == InputMode::ReactionPicker
        && let Some(rect) = selected_canvas_rect(state, canvas_area)
    {
        reaction_picker::render(frame, rect, state.reaction_picker_index, area, theme);
    }
}

fn selected_canvas_rect(state: &RenderState, canvas_area: Rect) -> Option<Rect> {
    let idx = state.selected?;
    let confession = state.confessions.get(idx)?;
    let screen_x = confession.x - state.cam_x;
    let screen_y = confession.y - state.cam_y;
    if screen_x < 0 || screen_y < 0 {
        return None;
    }

    let sx = screen_x as u16;
    let sy = screen_y as u16;
    if sx >= canvas_area.width || sy >= canvas_area.height {
        return None;
    }

    let box_h = confession::confession_height(&confession.text);
    let avail_w = canvas_area.width.saturating_sub(sx).min(BOX_WIDTH);
    let avail_h = canvas_area.height.saturating_sub(sy).min(box_h);
    if avail_w < 6 || avail_h < 3 {
        return None;
    }

    Some(Rect::new(
        canvas_area.x + sx,
        canvas_area.y + sy,
        avail_w,
        avail_h,
    ))
}
