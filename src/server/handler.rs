use std::sync::Arc;
use std::sync::atomic::Ordering;

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use log::{debug, info, warn};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use russh::server::{self, Auth, Msg, Session};
use russh::{Channel, ChannelId};
use sha2::{Digest, Sha256};

use crate::db;
use crate::model::confession::{self, Confession};
use crate::model::reaction;
use crate::model::reply::{self, Reply};
use crate::tui::canvas;
use crate::tui::{RenderState, TermWriter};

use super::AppState;
use super::input::{InputMode, KeyEvent};

pub(crate) struct ClientHandler {
    pub(crate) shared: Arc<AppState>,
    fingerprint: Option<String>,
    shell_channel: Option<ChannelId>,
    cam_x: i64,
    cam_y: i64,
    selected: Option<usize>,
    mode: InputMode,
    compose_buf: String,
    reply_name_buf: String,
    reply_name_phase: bool,
    width: u16,
    height: u16,
    confessions: Vec<Confession>,
    replies: Vec<Reply>,
    viewing_confession: Option<usize>,
    reply_scroll: usize,
    card_index: usize,
    came_from_card: bool,
    created_confession: bool,
    message: Option<String>,
    help_return_mode: InputMode,
    reaction_return_mode: InputMode,
    reaction_picker_index: usize,
    last_known_count: usize,
    search_buf: String,
    search_results: Vec<usize>,
    search_index: usize,
    splash_frame: u8,
    splash_done: Arc<std::sync::atomic::AtomicBool>,
    theme_index: usize,
    theme_picker_index: usize,
    terminal: Option<Terminal<CrosstermBackend<TermWriter>>>,
    writer: TermWriter,
}

impl ClientHandler {
    pub(crate) fn new(shared: Arc<AppState>) -> Self {
        Self {
            shared,
            fingerprint: None,
            shell_channel: None,
            cam_x: 0,
            cam_y: 0,
            selected: None,
            mode: InputMode::Splash,
            compose_buf: String::new(),
            reply_name_buf: String::new(),
            reply_name_phase: false,
            width: 80,
            height: 24,
            confessions: Vec::new(),
            replies: Vec::new(),
            viewing_confession: None,
            reply_scroll: 0,
            card_index: 0,
            came_from_card: false,
            created_confession: false,
            message: None,
            help_return_mode: InputMode::Browse,
            reaction_return_mode: InputMode::Browse,
            reaction_picker_index: 0,
            last_known_count: 0,
            search_buf: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            splash_frame: 0,
            splash_done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            theme_index: 0,
            theme_picker_index: 0,
            terminal: None,
            writer: TermWriter::default(),
        }
    }

    fn return_mode(&self) -> InputMode {
        if self.came_from_card {
            InputMode::CardView
        } else {
            InputMode::Browse
        }
    }

    fn fingerprint_str(&self) -> &str {
        self.fingerprint.as_deref().unwrap_or("unknown")
    }

    fn reload_confessions(&mut self) {
        let old_count = self.confessions.len();
        let db = self.shared.db.lock();
        self.confessions = db::get_all(&db);
        drop(db);

        let new_count = self.confessions.len();
        if self.last_known_count > 0 && new_count > old_count {
            let diff = new_count - old_count;
            self.message = Some(format!(
                "{} new confession{}!",
                diff,
                if diff == 1 { "" } else { "s" }
            ));
        }
        self.last_known_count = new_count;
    }

    fn visible_indices(&self) -> Vec<usize> {
        canvas::visible_confessions(
            &self.confessions,
            self.cam_x,
            self.cam_y,
            self.width,
            self.height.saturating_sub(1),
        )
    }

    fn cycle_selection(&mut self) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            self.selected = None;
            return;
        }

        self.selected = match self.selected {
            None => Some(visible[0]),
            Some(current) => {
                let pos = visible.iter().position(|&i| i == current);
                match pos {
                    Some(p) => Some(visible[(p + 1) % visible.len()]),
                    None => Some(visible[0]),
                }
            }
        };
    }

    fn select_at_screen(&mut self, sx: u16, sy: u16) {
        let world_x = sx as i64 + self.cam_x;
        let world_y = sy as i64 + self.cam_y;

        for (i, c) in self.confessions.iter().enumerate() {
            let bw = confession::BOX_WIDTH as i64;
            let bh = confession::confession_height(&c.text) as i64;
            if world_x >= c.x && world_x < c.x + bw && world_y >= c.y && world_y < c.y + bh {
                self.selected = Some(i);
                return;
            }
        }
    }

    fn open_replies(&mut self) {
        let Some(idx) = self.selected else { return };
        let Some(confession) = self.confessions.get(idx) else {
            return;
        };
        let db = self.shared.db.lock();
        self.replies = db::get_replies(&db, confession.id);
        self.viewing_confession = Some(idx);
        self.reply_scroll = 0;
        self.mode = InputMode::ViewReplies;
    }

    fn reload_replies(&mut self) {
        if let Some(idx) = self.viewing_confession
            && let Some(confession) = self.confessions.get(idx)
        {
            let db = self.shared.db.lock();
            self.replies = db::get_replies(&db, confession.id);
        }
    }

    fn submit_reply(&mut self) {
        let text = self.compose_buf.trim().to_string();
        if text.is_empty() {
            self.message = Some("Empty reply".to_string());
            return;
        }
        if text.len() > reply::MAX_LENGTH {
            self.message = Some("Too long (max 100 chars)".to_string());
            return;
        }
        if !confession::is_allowed(&text) {
            self.message = Some("Reply contains blocked words".to_string());
            return;
        }

        let Some(idx) = self.viewing_confession else {
            return;
        };
        let Some(confession) = self.confessions.get(idx) else {
            return;
        };

        let fp = self.fingerprint_str();
        let name = if self.reply_name_buf.trim().is_empty() {
            None
        } else {
            Some(self.reply_name_buf.trim().to_string())
        };

        let db = self.shared.db.lock();
        match db::insert_reply(&db, confession.id, &text, name.as_deref(), fp) {
            Ok(_) => {
                self.message = Some("Reply posted!".to_string());
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }
        drop(db);

        self.compose_buf.clear();
        self.reply_name_buf.clear();
        self.reload_replies();
        self.reload_confessions();
        self.mode = InputMode::ViewReplies;
    }

    fn submit_confession(&mut self) {
        let text = self.compose_buf.trim().to_string();
        if text.is_empty() {
            self.message = Some("Empty confession".to_string());
            return;
        }
        if text.len() > confession::MAX_LENGTH {
            self.message = Some("Too long (max 280 chars)".to_string());
            return;
        }
        if !confession::is_allowed(&text) {
            self.message = Some("Confession contains blocked words".to_string());
            return;
        }

        let fp = self.fingerprint_str().to_owned();
        let db = self.shared.db.lock();

        if std::env::var("EIPI_NO_LIMIT").is_err() {
            let today = db::posts_today(&db, &fp);
            if today >= crate::helper::consts::DAILY_POST_LIMIT {
                self.message = Some(format!(
                    "Rate limit: {} confessions per day",
                    crate::helper::consts::DAILY_POST_LIMIT
                ));
                return;
            }
        }

        drop(db);
        self.reload_confessions();
        let (x, y) = canvas::random_position(&self.confessions, &text);
        let db = self.shared.db.lock();

        match db::insert(&db, &text, x, y, &fp) {
            Ok(_) => {
                self.created_confession = true;
                self.message = Some("Confession posted!".to_string());
                self.cam_x = x - self.width as i64 / 2;
                self.cam_y = y - self.height as i64 / 2;
                drop(db);
                self.shared.notify.send(()).ok();
            }
            Err(e) => {
                drop(db);
                self.message = Some(format!("Error: {}", e));
            }
        }

        self.compose_buf.clear();
        self.reload_confessions();
    }

    fn handle_help_event(&mut self, event: &KeyEvent) {
        if matches!(
            event,
            KeyEvent::Escape | KeyEvent::Char('q') | KeyEvent::Char('?') | KeyEvent::Enter
        ) {
            self.mode = self.help_return_mode;
        }
    }

    fn clear_compose(&mut self) {
        self.compose_buf.clear();
    }

    fn clear_reply_compose(&mut self) {
        self.compose_buf.clear();
        self.reply_name_buf.clear();
    }

    fn clear_search(&mut self) {
        self.search_buf.clear();
    }

    fn clear_search_results(&mut self) {
        self.search_results.clear();
        self.search_buf.clear();
    }

    fn close_replies(&mut self) {
        self.mode = self.return_mode();
        self.came_from_card = false;
        self.viewing_confession = None;
        self.replies.clear();
    }

    fn open_compose(&mut self, came_from_card: bool) {
        self.came_from_card = came_from_card;
        self.mode = InputMode::Compose;
        self.clear_compose();
    }

    fn open_search(&mut self) {
        self.mode = InputMode::Search;
        self.clear_search();
    }

    fn open_theme_picker(&mut self) {
        self.theme_picker_index = self.theme_index;
        self.mode = InputMode::ThemePicker;
    }

    fn open_help(&mut self) {
        self.help_return_mode = self.mode;
        self.mode = InputMode::Help;
    }

    fn open_reply_compose(&mut self) {
        self.mode = InputMode::ComposeReply;
        self.clear_reply_compose();
        self.reply_name_phase = true;
    }

    fn selected_confession_id(&self) -> Option<i64> {
        let mode = if self.mode == InputMode::ReactionPicker {
            self.reaction_return_mode
        } else {
            self.mode
        };

        match mode {
            InputMode::Browse => self
                .selected
                .and_then(|idx| self.confessions.get(idx))
                .map(|confession| confession.id),
            InputMode::CardView | InputMode::SearchResults => self
                .confessions
                .get(self.card_index)
                .map(|confession| confession.id),
            InputMode::ViewReplies => self
                .viewing_confession
                .and_then(|idx| self.confessions.get(idx))
                .map(|confession| confession.id),
            _ => None,
        }
    }

    fn show_share_link(&mut self) {
        let base =
            std::env::var("EIPI_BASE_URL").unwrap_or_else(|_| "https://eipi.boo".to_string());
        if let Some(id) = self.selected_confession_id() {
            self.message = Some(format!("{}/c/{}", base, id));
        }
    }

    fn open_reaction_picker(&mut self) {
        let Some(confession_id) = self.selected_confession_id() else {
            return;
        };

        self.reaction_picker_index = {
            let db = self.shared.db.lock();
            db::get_reaction(&db, confession_id, self.fingerprint_str())
                .ok()
                .flatten()
                .and_then(|token| reaction::find_index(&token))
                .unwrap_or(0)
        };
        self.reaction_return_mode = self.mode;
        self.mode = InputMode::ReactionPicker;
    }

    fn submit_reaction(&mut self) {
        let Some(confession_id) = self.selected_confession_id() else {
            return;
        };

        let token = reaction::token_at(self.reaction_picker_index);
        let db = self.shared.db.lock();
        match db::set_reaction(&db, confession_id, token, self.fingerprint_str()) {
            Ok(()) => self.message = Some(format!("reacted with {}", token)),
            Err(err) => self.message = Some(format!("reaction failed: {}", err)),
        }
        drop(db);

        self.reload_confessions();
        self.mode = self.reaction_return_mode;
    }

    fn handle_browse_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Char('q') => self.mode = InputMode::ConfirmQuit,
            KeyEvent::Up | KeyEvent::Char('k') => self.cam_y -= crate::helper::consts::CAM_SPEED_Y,
            KeyEvent::Down | KeyEvent::Char('j') => {
                self.cam_y += crate::helper::consts::CAM_SPEED_Y
            }
            KeyEvent::Left | KeyEvent::Char('h') => {
                self.cam_x -= crate::helper::consts::CAM_SPEED_X
            }
            KeyEvent::Right | KeyEvent::Char('l') => {
                self.cam_x += crate::helper::consts::CAM_SPEED_X
            }
            KeyEvent::Tab => self.cycle_selection(),
            KeyEvent::Enter => {
                self.came_from_card = false;
                self.open_replies();
            }
            KeyEvent::Char('n') => self.open_compose(false),
            KeyEvent::MouseClick(sx, sy) => self.select_at_screen(*sx, *sy),
            KeyEvent::Char(' ') if !self.confessions.is_empty() => {
                self.card_index = self.selected.unwrap_or(0);
                self.mode = InputMode::CardView;
            }
            KeyEvent::Char('G') => {
                if let Some(last_idx) = self.confessions.len().checked_sub(1) {
                    let last = &self.confessions[last_idx];
                    self.cam_x = last.x - self.width as i64 / 2;
                    self.cam_y = last.y - self.height as i64 / 2;
                    self.selected = Some(last_idx);
                }
            }
            KeyEvent::Char('/') => self.open_search(),
            KeyEvent::Char('T') => self.open_theme_picker(),
            KeyEvent::Char('?') => self.open_help(),
            KeyEvent::Char('f') => self.open_reaction_picker(),
            KeyEvent::Char('s') => self.show_share_link(),
            _ => {}
        }
    }

    fn handle_card_view_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Right | KeyEvent::Char('l') if !self.confessions.is_empty() => {
                self.card_index = (self.card_index + 1) % self.confessions.len();
                self.selected = Some(self.card_index);
            }
            KeyEvent::Left | KeyEvent::Char('h') if !self.confessions.is_empty() => {
                self.card_index = if self.card_index == 0 {
                    self.confessions.len() - 1
                } else {
                    self.card_index - 1
                };
                self.selected = Some(self.card_index);
            }
            KeyEvent::MouseClick(sx, _) if !self.confessions.is_empty() => {
                if *sx < self.width / 2 {
                    self.card_index = if self.card_index == 0 {
                        self.confessions.len() - 1
                    } else {
                        self.card_index - 1
                    };
                } else {
                    self.card_index = (self.card_index + 1) % self.confessions.len();
                }
                self.selected = Some(self.card_index);
            }
            KeyEvent::Enter => {
                self.selected = Some(self.card_index);
                self.came_from_card = true;
                self.open_replies();
            }
            KeyEvent::Char('n') => self.open_compose(true),
            KeyEvent::Escape | KeyEvent::Char('q') | KeyEvent::Char(' ') => {
                self.selected = Some(self.card_index);
                self.mode = InputMode::Browse;
            }
            KeyEvent::Char('/') => self.open_search(),
            KeyEvent::Char('T') => self.open_theme_picker(),
            KeyEvent::Char('?') => self.open_help(),
            KeyEvent::Char('f') => self.open_reaction_picker(),
            KeyEvent::Char('s') => self.show_share_link(),
            _ => {}
        }
    }

    fn handle_theme_picker_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Up | KeyEvent::Char('k') => {
                let count = crate::tui::themes::ALL.len();
                self.theme_picker_index = if self.theme_picker_index == 0 {
                    count - 1
                } else {
                    self.theme_picker_index - 1
                };
                self.theme_index = self.theme_picker_index;
            }
            KeyEvent::Down | KeyEvent::Char('j') => {
                let count = crate::tui::themes::ALL.len();
                self.theme_picker_index = (self.theme_picker_index + 1) % count;
                self.theme_index = self.theme_picker_index;
            }
            KeyEvent::Enter => {
                self.theme_index = self.theme_picker_index;
                let name = crate::tui::themes::ALL[self.theme_index].0;
                self.message = Some(format!("theme: {}", name));
                let db = self.shared.db.lock();
                db::set_theme(&db, self.fingerprint_str(), name);
                drop(db);
                self.mode = self.return_mode();
            }
            KeyEvent::Escape => self.mode = self.return_mode(),
            _ => {}
        }
    }

    fn handle_confirm_quit_event(&mut self, event: &KeyEvent) -> bool {
        match event {
            KeyEvent::Char('q') | KeyEvent::Enter => true,
            _ => {
                self.mode = InputMode::Browse;
                false
            }
        }
    }

    fn handle_reaction_picker_event(&mut self, event: &KeyEvent) {
        let total = reaction::ALL.len();
        match event {
            KeyEvent::Escape => self.mode = self.reaction_return_mode,
            KeyEvent::Enter => self.submit_reaction(),
            KeyEvent::Left | KeyEvent::Char('h') => {
                self.reaction_picker_index = self.reaction_picker_index.saturating_sub(1);
            }
            KeyEvent::Right | KeyEvent::Char('l') if self.reaction_picker_index + 1 < total => {
                self.reaction_picker_index += 1;
            }
            KeyEvent::Up | KeyEvent::Char('k') => {
                self.reaction_picker_index = self.reaction_picker_index.saturating_sub(4);
            }
            KeyEvent::Down | KeyEvent::Char('j') => {
                self.reaction_picker_index = (self.reaction_picker_index + 4).min(total - 1);
            }
            _ => {}
        }
    }

    fn handle_view_replies_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Escape | KeyEvent::Char('q') => self.close_replies(),
            KeyEvent::Up | KeyEvent::Char('k') => {
                self.reply_scroll = self.reply_scroll.saturating_sub(1);
            }
            KeyEvent::Down | KeyEvent::Char('j') if self.reply_scroll + 1 < self.replies.len() => {
                self.reply_scroll += 1;
            }
            KeyEvent::Char('r') => self.open_reply_compose(),
            KeyEvent::Char('?') => self.open_help(),
            KeyEvent::Char('f') => self.open_reaction_picker(),
            KeyEvent::Char('s') => self.show_share_link(),
            _ => {}
        }
    }

    fn handle_compose_reply_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Escape => {
                self.mode = InputMode::ViewReplies;
                self.clear_reply_compose();
            }
            KeyEvent::Enter => {
                if self.reply_name_phase {
                    self.reply_name_phase = false;
                } else {
                    self.submit_reply();
                }
            }
            KeyEvent::Char(c) => {
                if self.reply_name_phase {
                    if self.reply_name_buf.len() < crate::helper::consts::MAX_REPLY_NAME_LENGTH {
                        self.reply_name_buf.push(*c);
                    }
                } else if self.compose_buf.len() < reply::MAX_LENGTH {
                    self.compose_buf.push(*c);
                }
            }
            KeyEvent::Backspace => {
                if self.reply_name_phase {
                    self.reply_name_buf.pop();
                } else {
                    self.compose_buf.pop();
                }
            }
            _ => {}
        }
    }

    fn handle_search_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Escape => {
                self.mode = self.return_mode();
                self.clear_search();
            }
            KeyEvent::Enter => {
                let query = self.search_buf.to_lowercase();
                if query.is_empty() {
                    self.mode = self.return_mode();
                } else {
                    self.search_results = self
                        .confessions
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.text.to_lowercase().contains(&query))
                        .map(|(i, _)| i)
                        .collect();
                    if self.search_results.is_empty() {
                        self.message = Some(format!("No results for \"{}\"", self.search_buf));
                        self.mode = self.return_mode();
                    } else {
                        self.search_index = 0;
                        self.card_index = self.search_results[0];
                        self.selected = Some(self.card_index);
                        self.mode = InputMode::SearchResults;
                    }
                }
            }
            KeyEvent::Char(c) => self.search_buf.push(*c),
            KeyEvent::Backspace => {
                self.search_buf.pop();
            }
            _ => {}
        }
    }

    fn handle_search_results_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Right | KeyEvent::Char('l') if !self.search_results.is_empty() => {
                self.search_index = (self.search_index + 1) % self.search_results.len();
                self.card_index = self.search_results[self.search_index];
                self.selected = Some(self.card_index);
            }
            KeyEvent::Left | KeyEvent::Char('h') if !self.search_results.is_empty() => {
                self.search_index = if self.search_index == 0 {
                    self.search_results.len() - 1
                } else {
                    self.search_index - 1
                };
                self.card_index = self.search_results[self.search_index];
                self.selected = Some(self.card_index);
            }
            KeyEvent::Enter => {
                self.selected = Some(self.card_index);
                self.came_from_card = true;
                self.open_replies();
            }
            KeyEvent::Escape | KeyEvent::Char('q') => {
                self.clear_search_results();
                self.mode = InputMode::Browse;
            }
            KeyEvent::Char('?') => self.open_help(),
            KeyEvent::Char('f') => self.open_reaction_picker(),
            KeyEvent::Char('s') => self.show_share_link(),
            _ => {}
        }
    }

    fn handle_compose_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Escape => {
                self.mode = self.return_mode();
                self.clear_compose();
            }
            KeyEvent::Enter => {
                self.submit_confession();
                self.mode = self.return_mode();
            }
            KeyEvent::Char(c) if self.compose_buf.len() < confession::MAX_LENGTH => {
                self.compose_buf.push(*c);
            }
            KeyEvent::Backspace => {
                self.compose_buf.pop();
            }
            _ => {}
        }
    }

    fn process_input(&mut self, events: Vec<KeyEvent>) -> bool {
        for event in events {
            if self.message.is_some()
                && matches!(
                    self.mode,
                    InputMode::Browse
                        | InputMode::CardView
                        | InputMode::SearchResults
                        | InputMode::ViewReplies
                )
                && event != KeyEvent::Char('q')
                && event != KeyEvent::Char('s')
            {
                self.message = None;
            }

            match self.mode {
                InputMode::Help => self.handle_help_event(&event),
                InputMode::Browse => self.handle_browse_event(&event),
                InputMode::CardView => self.handle_card_view_event(&event),
                InputMode::ThemePicker => self.handle_theme_picker_event(&event),
                InputMode::ReactionPicker => self.handle_reaction_picker_event(&event),
                InputMode::ConfirmQuit => {
                    if self.handle_confirm_quit_event(&event) {
                        return true;
                    }
                }
                InputMode::ViewReplies => self.handle_view_replies_event(&event),
                InputMode::ComposeReply => self.handle_compose_reply_event(&event),
                InputMode::Search => self.handle_search_event(&event),
                InputMode::SearchResults => self.handle_search_results_event(&event),
                InputMode::Compose => self.handle_compose_event(&event),
                InputMode::Splash => {}
            }
        }
        false
    }

    fn do_render(&mut self) -> Vec<u8> {
        let Some(terminal) = self.terminal.as_mut() else {
            debug!("do_render: no terminal initialized");
            return Vec::new();
        };
        let (total_confessions, total_humans) = {
            let db = self.shared.db.lock();
            let stats = db::stats(&db);
            (stats.confessions, stats.humans)
        };

        let viewing = self
            .viewing_confession
            .and_then(|i| self.confessions.get(i));

        let state = RenderState {
            confessions: &self.confessions,
            cam_x: self.cam_x,
            cam_y: self.cam_y,
            selected: self.selected,
            mode: self.mode,
            reaction_return_mode: self.reaction_return_mode,
            compose_buf: &self.compose_buf,
            reply_name_buf: &self.reply_name_buf,
            reply_name_phase: self.reply_name_phase,
            message: self.message.as_deref(),
            total_confessions,
            total_humans,
            online: self.shared.online.load(Ordering::Relaxed),
            replies: &self.replies,
            viewing_confession: viewing,
            reply_scroll: self.reply_scroll,
            card_index: self.card_index,
            came_from_card: self.came_from_card,
            created_confession: self.created_confession,
            search_buf: &self.search_buf,
            search_result_count: self.search_results.len(),
            search_index: self.search_index,
            splash_frame: self.splash_frame,
            theme: crate::tui::themes::ALL[self.theme_index].1,
            theme_picker_index: self.theme_picker_index,
            reaction_picker_index: self.reaction_picker_index,
            render_tick: chrono::Utc::now().timestamp() as u64,
        };

        match terminal.draw(|frame| {
            crate::tui::render(frame, &state);
        }) {
            Ok(_) => {}
            Err(e) => warn!("Render error: {}", e),
        }

        self.writer.drain()
    }

    fn init_terminal(&mut self) -> Vec<u8> {
        crossterm::execute!(
            self.writer,
            EnterAlternateScreen,
            cursor::Hide,
            EnableMouseCapture
        )
        .ok();
        let init_bytes = self.writer.drain();

        match crate::tui::create_terminal(self.writer.clone(), self.width, self.height) {
            Ok(t) => {
                self.terminal = Some(t);
                debug!("Terminal initialized: {}x{}", self.width, self.height);
            }
            Err(e) => warn!("Failed to create terminal: {}", e),
        }

        init_bytes
    }

    fn theme_osc_bytes(&self) -> Vec<u8> {
        crate::tui::themes::ALL[self.theme_index].1.osc_bytes()
    }

    fn cleanup_bytes(&mut self) -> Vec<u8> {
        crossterm::execute!(
            self.writer,
            DisableMouseCapture,
            cursor::Show,
            LeaveAlternateScreen
        )
        .ok();
        let mut bytes = crate::tui::theme::Theme::osc_reset();
        bytes.extend(self.writer.drain());
        bytes
    }
}

impl server::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn auth_publickey_offered(
        &mut self,
        _user: &str,
        _key: &russh::keys::PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        key: &russh::keys::PublicKey,
    ) -> Result<Auth, Self::Error> {
        let raw_fp = key.fingerprint(russh::keys::HashAlg::Sha256);
        let hash = Sha256::digest(raw_fp.as_bytes());
        let hashed_fp = format!("{:x}", hash);
        self.fingerprint = Some(hashed_fp);
        info!("Auth accepted (fingerprint hashed)");
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        debug!("Channel opened: {:?}", channel.id());
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        channel_id: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.width = if col_width > 0 { col_width as u16 } else { 80 };
        self.height = if row_height > 0 {
            row_height as u16
        } else {
            24
        };
        debug!(
            "PTY request on channel {:?}: {}x{}",
            channel_id, self.width, self.height
        );
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel_id: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Ghostty and some terminals send exec requests (e.g. terminfo setup)
        // before the shell. Reject and close so they move on to the shell channel.
        let cmd = String::from_utf8_lossy(data);
        debug!("Exec request on channel {:?}: {}", channel_id, cmd);
        let _ = session.channel_failure(channel_id);
        let _ = session.close(channel_id);
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel_id: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!("Shell request on channel {:?}", channel_id);
        self.shell_channel = Some(channel_id);

        self.shared.online.fetch_add(1, Ordering::Relaxed);
        self.reload_confessions();

        // load saved theme preference
        if let Some(saved) = {
            let db = self.shared.db.lock();
            db::get_theme(&db, self.fingerprint_str())
        } && let Some(idx) = crate::tui::themes::ALL
            .iter()
            .position(|(name, _)| *name == saved)
        {
            self.theme_index = idx;
        }

        let (cx, cy) = canvas::best_camera(&self.confessions, self.width, self.height);
        self.cam_x = cx;
        self.cam_y = cy;

        let init = self.init_terminal();
        let _ = session.data(channel_id, init);

        // set terminal bg/fg colors
        let osc = self.theme_osc_bytes();
        let _ = session.data(channel_id, osc);

        let visible = self.visible_indices();
        if !visible.is_empty() {
            self.selected = Some(visible[0]);
        }

        let output = self.do_render();
        if !output.is_empty() {
            debug!("Initial render: {} bytes", output.len());
            let _ = session.data(channel_id, output);
        }

        // splash animation: advance frames on a timer
        {
            let handle = session.handle();
            let shared = self.shared.clone();
            let width = self.width;
            let height = self.height;
            let confessions = self.confessions.clone();
            let writer = self.writer.clone();
            let splash_done = self.splash_done.clone();
            let theme_index = self.theme_index;
            tokio::spawn(async move {
                use crate::tui::splash::TOTAL_FRAMES;
                for f in 1..TOTAL_FRAMES {
                    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                    if splash_done.load(Ordering::Relaxed) {
                        break;
                    }
                    let w = writer.clone();
                    let term = crate::tui::create_terminal(w.clone(), width, height);
                    let Ok(mut term) = term else { break };
                    let state = RenderState {
                        confessions: &confessions,
                        cam_x: 0,
                        cam_y: 0,
                        selected: None,
                        mode: InputMode::Splash,
                        reaction_return_mode: InputMode::Browse,
                        compose_buf: "",
                        reply_name_buf: "",
                        reply_name_phase: false,
                        message: None,
                        total_confessions: 0,
                        total_humans: 0,
                        online: shared.online.load(Ordering::Relaxed),
                        replies: &[],
                        viewing_confession: None,
                        reply_scroll: 0,
                        card_index: 0,
                        came_from_card: false,
                        created_confession: false,
                        search_buf: "",
                        search_result_count: 0,
                        search_index: 0,
                        splash_frame: f,
                        theme: crate::tui::themes::ALL[theme_index].1,
                        theme_picker_index: 0,
                        reaction_picker_index: 0,
                        render_tick: chrono::Utc::now().timestamp() as u64,
                    };
                    let _ = term.draw(|frame| {
                        crate::tui::render(frame, &state);
                    });
                    let bytes = w.drain();
                    if bytes.is_empty() {
                        continue;
                    }
                    if handle.data(channel_id, bytes).await.is_err() {
                        break;
                    }
                }
                // animation finished, clear screen and render Browse view
                splash_done.store(true, Ordering::Relaxed);

                // clear screen first
                {
                    let mut w = writer.clone();
                    crossterm::execute!(
                        w,
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                    )
                    .ok();
                    let clear_bytes = w.drain();
                    if !clear_bytes.is_empty() {
                        let _ = handle.data(channel_id, clear_bytes).await;
                    }
                }

                let confessions = {
                    let db = shared.db.lock();
                    crate::db::get_all(&db)
                };
                let (cam_x, cam_y) = crate::tui::canvas::best_camera(&confessions, width, height);
                let visible = crate::tui::canvas::visible_confessions(
                    &confessions,
                    cam_x,
                    cam_y,
                    width,
                    height.saturating_sub(1),
                );
                let selected = visible.first().copied();
                let (total_confessions, total_humans) = {
                    let db = shared.db.lock();
                    let stats = crate::db::stats(&db);
                    (stats.confessions, stats.humans)
                };

                let w = writer.clone();
                let term = crate::tui::create_terminal(w.clone(), width, height);
                if let Ok(mut term) = term {
                    let state = RenderState {
                        confessions: &confessions,
                        cam_x,
                        cam_y,
                        selected,
                        mode: InputMode::Browse,
                        reaction_return_mode: InputMode::Browse,
                        compose_buf: "",
                        reply_name_buf: "",
                        reply_name_phase: false,
                        message: None,
                        total_confessions,
                        total_humans,
                        online: shared.online.load(Ordering::Relaxed),
                        replies: &[],
                        viewing_confession: None,
                        reply_scroll: 0,
                        card_index: 0,
                        came_from_card: false,
                        created_confession: false,
                        search_buf: "",
                        search_result_count: 0,
                        search_index: 0,
                        splash_frame: 0,
                        theme: crate::tui::themes::ALL[theme_index].1,
                        theme_picker_index: 0,
                        reaction_picker_index: 0,
                        render_tick: chrono::Utc::now().timestamp() as u64,
                    };
                    let _ = term.draw(|frame| {
                        crate::tui::render(frame, &state);
                    });
                    let bytes = w.drain();
                    if !bytes.is_empty() {
                        let _ = handle.data(channel_id, bytes).await;
                    }
                }
            });
        }

        // background task: notify this client when someone else posts
        let handle = session.handle();
        let mut rx = self.shared.notify.subscribe();
        tokio::spawn(async move {
            while rx.recv().await.is_ok() {
                // bell character makes the terminal flash/beep
                if handle.data(channel_id, "\x07".as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        Ok(())
    }

    async fn data(
        &mut self,
        channel_id: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.shell_channel != Some(channel_id) {
            debug!(
                "Ignoring data on non-shell channel {:?} ({} bytes)",
                channel_id,
                data.len()
            );
            return Ok(());
        }

        // auto-transition from splash when animation finishes
        if self.mode == InputMode::Splash {
            if self.splash_done.load(Ordering::Relaxed) {
                self.mode = InputMode::Browse;
                // clear screen and recreate terminal to remove splash artifacts
                crossterm::execute!(self.writer, terminal::Clear(terminal::ClearType::All)).ok();
                let clear_bytes = self.writer.drain();
                if !clear_bytes.is_empty() {
                    let _ = session.data(channel_id, clear_bytes);
                }
                self.terminal =
                    crate::tui::create_terminal(self.writer.clone(), self.width, self.height).ok();
                self.reload_confessions();
                let (cx, cy) = canvas::best_camera(&self.confessions, self.width, self.height);
                self.cam_x = cx;
                self.cam_y = cy;
                let visible = self.visible_indices();
                if !visible.is_empty() {
                    self.selected = Some(visible[0]);
                }
                let output = self.do_render();
                if !output.is_empty() {
                    let _ = session.data(channel_id, output);
                }
                return Ok(());
            } else {
                // swallow all input during splash
                return Ok(());
            }
        }

        let events = super::input::parse(data);
        if events.is_empty() {
            return Ok(());
        }

        let prev_theme = self.theme_index;
        let should_quit = self.process_input(events);

        if should_quit {
            let cleanup = self.cleanup_bytes();
            if !cleanup.is_empty() {
                let _ = session.data(channel_id, cleanup);
            }
            let _ = session.close(channel_id);
            return Ok(());
        }

        // send OSC to change terminal colors when theme changes
        if self.theme_index != prev_theme {
            let osc = self.theme_osc_bytes();
            let _ = session.data(channel_id, osc);
        }

        self.reload_confessions();

        let output = self.do_render();
        if !output.is_empty() {
            let _ = session.data(channel_id, output);
        }

        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel_id: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("Window change: {}x{}", col_width, row_height);
        self.width = col_width as u16;
        self.height = row_height as u16;

        crossterm::execute!(self.writer, terminal::Clear(terminal::ClearType::All)).ok();
        let clear_bytes = self.writer.drain();
        if !clear_bytes.is_empty() {
            let _ = session.data(channel_id, clear_bytes);
        }

        self.terminal =
            crate::tui::create_terminal(self.writer.clone(), self.width, self.height).ok();

        let output = self.do_render();
        if !output.is_empty() {
            let _ = session.data(channel_id, output);
        }

        Ok(())
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        if self.shell_channel.is_some() {
            self.shared.online.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
