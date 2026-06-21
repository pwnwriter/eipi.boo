use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use crossterm::cursor;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use log::{debug, info, warn};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use russh::server::{self, Auth, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec};

use crate::canvas;
use crate::confession::{self, Confession};
use crate::db;
use crate::input::{InputMode, KeyEvent};
use crate::server::AppState;
use crate::tui::{RenderState, TermWriter};

pub(crate) struct ClientHandler {
    pub(crate) shared: Arc<AppState>,
    fingerprint: Option<String>,
    shell_channel: Option<ChannelId>,
    cam_x: i64,
    cam_y: i64,
    selected: Option<usize>,
    mode: InputMode,
    compose_buf: String,
    width: u16,
    height: u16,
    confessions: Vec<Confession>,
    message: Option<String>,
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
            mode: InputMode::Browse,
            compose_buf: String::new(),
            width: 80,
            height: 24,
            confessions: Vec::new(),
            message: None,
            terminal: None,
            writer: TermWriter::default(),
        }
    }

    fn fingerprint_str(&self) -> String {
        self.fingerprint
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn reload_confessions(&mut self) {
        let db = self.shared.db.lock().unwrap();
        self.confessions = db::get_all(&db);
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

    fn upvote_selected(&mut self) {
        let Some(idx) = self.selected else { return };
        let Some(confession) = self.confessions.get(idx) else {
            return;
        };
        let confession_id = confession.id;
        let fp = self.fingerprint_str();

        let db = self.shared.db.lock().unwrap();
        match db::upvote(&db, confession_id, &fp) {
            Ok(new_votes) => {
                self.message = Some(format!("Upvoted! (♥ {})", new_votes));
            }
            Err(e) => {
                self.message = Some(format!("Can't vote: {}", e));
            }
        }
        drop(db);
        self.reload_confessions();
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

        let fp = self.fingerprint_str();
        let db = self.shared.db.lock().unwrap();

        let today = db::posts_today(&db, &fp);
        if today >= db::DAILY_POST_LIMIT {
            self.message = Some(format!(
                "Rate limit: {} confessions per day",
                db::DAILY_POST_LIMIT
            ));
            return;
        }

        drop(db);
        self.reload_confessions();
        let (x, y) = canvas::random_position(&self.confessions, &text);
        let db = self.shared.db.lock().unwrap();

        match db::insert(&db, &text, x, y, &fp) {
            Ok(_) => {
                self.message = Some("Confession posted!".to_string());
                self.cam_x = x - self.width as i64 / 2;
                self.cam_y = y - self.height as i64 / 2;
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }

        drop(db);
        self.compose_buf.clear();
        self.reload_confessions();
    }

    fn process_input(&mut self, events: Vec<KeyEvent>) -> bool {
        for event in events {
            if self.message.is_some()
                && self.mode == InputMode::Browse
                && event != KeyEvent::Char('q')
            {
                self.message = None;
            }

            match (&self.mode, &event) {
                (InputMode::Browse, KeyEvent::Char('q')) => return true,
                (InputMode::Browse, KeyEvent::Up | KeyEvent::Char('k')) => self.cam_y -= 3,
                (InputMode::Browse, KeyEvent::Down | KeyEvent::Char('j')) => self.cam_y += 3,
                (InputMode::Browse, KeyEvent::Left | KeyEvent::Char('h')) => self.cam_x -= 5,
                (InputMode::Browse, KeyEvent::Right | KeyEvent::Char('l')) => self.cam_x += 5,
                (InputMode::Browse, KeyEvent::Tab) => self.cycle_selection(),
                (InputMode::Browse, KeyEvent::Enter) => self.upvote_selected(),
                (InputMode::Browse, KeyEvent::Char('n')) => {
                    self.mode = InputMode::Compose;
                    self.compose_buf.clear();
                }
                (InputMode::Browse, KeyEvent::MouseClick(sx, sy)) => {
                    self.select_at_screen(*sx, *sy);
                }
                (InputMode::Browse, KeyEvent::Char('?')) => {
                    self.message = Some(
                        "bugs/features → https://github.com/pwnwriter/eipi.boo/issues/new"
                            .to_string(),
                    );
                }
                (InputMode::Compose, KeyEvent::Escape) => {
                    self.mode = InputMode::Browse;
                    self.compose_buf.clear();
                }
                (InputMode::Compose, KeyEvent::Enter) => {
                    self.submit_confession();
                    self.mode = InputMode::Browse;
                }
                (InputMode::Compose, KeyEvent::Char(c)) => {
                    if self.compose_buf.len() < confession::MAX_LENGTH {
                        self.compose_buf.push(*c);
                    }
                }
                (InputMode::Compose, KeyEvent::Backspace) => {
                    self.compose_buf.pop();
                }
                _ => {}
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
            let db = self.shared.db.lock().unwrap();
            db::stats(&db)
        };

        let state = RenderState {
            confessions: &self.confessions,
            cam_x: self.cam_x,
            cam_y: self.cam_y,
            selected: self.selected,
            mode: self.mode,
            compose_buf: &self.compose_buf,
            message: self.message.as_deref(),
            total_confessions,
            total_humans,
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
        crossterm::execute!(self.writer, EnterAlternateScreen, cursor::Hide, EnableMouseCapture).ok();
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

    fn cleanup_bytes(&mut self) -> Vec<u8> {
        crossterm::execute!(self.writer, DisableMouseCapture, cursor::Show, LeaveAlternateScreen).ok();
        self.writer.drain()
    }
}

#[async_trait]
impl server::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn auth_publickey_offered(
        &mut self,
        _user: &str,
        _key: &russh_keys::PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        key: &russh_keys::PublicKey,
    ) -> Result<Auth, Self::Error> {
        let fp = key.fingerprint(russh_keys::HashAlg::Sha256);
        self.fingerprint = Some(fp.to_string());
        info!("Auth accepted for fingerprint: {}", self.fingerprint_str());
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

        self.reload_confessions();

        let (cx, cy) = canvas::best_camera(&self.confessions, self.width, self.height);
        self.cam_x = cx;
        self.cam_y = cy;

        let init = self.init_terminal();
        let _ = session.data(channel_id, CryptoVec::from(init));

        let visible = self.visible_indices();
        if !visible.is_empty() {
            self.selected = Some(visible[0]);
        }

        let output = self.do_render();
        if !output.is_empty() {
            debug!("Initial render: {} bytes", output.len());
            let _ = session.data(channel_id, CryptoVec::from(output));
        }

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

        let events = crate::input::parse(data);
        if events.is_empty() {
            return Ok(());
        }

        let should_quit = self.process_input(events);

        if should_quit {
            let cleanup = self.cleanup_bytes();
            if !cleanup.is_empty() {
                let _ = session.data(channel_id, CryptoVec::from(cleanup));
            }
            let _ = session.close(channel_id);
            return Ok(());
        }

        self.reload_confessions();

        let output = self.do_render();
        if !output.is_empty() {
            let _ = session.data(channel_id, CryptoVec::from(output));
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

        if self.terminal.is_some() {
            self.terminal = None;
            if let Ok(t) = crate::tui::create_terminal(self.writer.clone(), self.width, self.height)
            {
                self.terminal = Some(t);
            }
        }

        let output = self.do_render();
        if !output.is_empty() {
            let _ = session.data(channel_id, CryptoVec::from(output));
        }

        Ok(())
    }
}
