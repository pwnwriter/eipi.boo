use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::helper::consts;
use crate::model::confession::{self, Confession};

const SPARKLES: [char; 4] = ['✦', '✧', '·', '*'];

fn sparkle_hash(x: i64, y: i64) -> u64 {
    let mut h = (x as u64).wrapping_mul(374761393);
    h = h.wrapping_add((y as u64).wrapping_mul(668265263));
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    h
}

/// Render a 2-ring sparkle glow around a screen-space rectangle.
pub fn render_ring(frame: &mut Frame, votes: i64, rect: Rect, clip: Rect) {
    let density = if votes > consts::VOTES_MAGENTA {
        consts::GLOW_DENSITY_HIGH
    } else {
        consts::GLOW_DENSITY_LOW
    };
    let color = if votes > consts::VOTES_MAGENTA {
        Color::Magenta
    } else {
        Color::Cyan
    };

    let buf = frame.buffer_mut();

    for ring in 1..=2u16 {
        let top = rect.y.saturating_sub(ring);
        let bot = rect.y + rect.height + ring - 1;
        let left = rect.x.saturating_sub(ring);
        let right = rect.x + rect.width + ring - 1;

        for y in top..=bot {
            for x in left..=right {
                let inside = x >= rect.x
                    && x < rect.x + rect.width
                    && y >= rect.y
                    && y < rect.y + rect.height;
                if inside {
                    continue;
                }

                if x < clip.x || x >= clip.x + clip.width || y < clip.y || y >= clip.y + clip.height
                {
                    continue;
                }

                let h = sparkle_hash(x as i64, y as i64);
                if !h.is_multiple_of(density) {
                    continue;
                }

                let cell = &buf[(x, y)];
                if cell.symbol() != " " {
                    continue;
                }

                let sparkle = SPARKLES[(h / 7 % SPARKLES.len() as u64) as usize];
                let style = if ring == 1 {
                    Style::default().fg(color)
                } else {
                    Style::default().fg(Color::Indexed(238))
                };

                let cell = &mut buf[(x, y)];
                cell.set_char(sparkle);
                cell.set_style(style);
            }
        }
    }
}

/// Render glow around popular confessions on the 2D canvas.
pub fn render(frame: &mut Frame, confessions: &[Confession], cam_x: i64, cam_y: i64, area: Rect) {
    for c in confessions {
        if c.votes <= consts::VOTES_GLOW {
            continue;
        }

        let box_h = confession::confession_height(&c.text);
        let box_w = consts::BOX_WIDTH;

        let sx = c.x - cam_x;
        let sy = c.y - cam_y;

        if sx + box_w as i64 <= 0 || sy + box_h as i64 <= 0 {
            continue;
        }

        let screen_rect = Rect::new(
            area.x.saturating_add_signed(sx as i16),
            area.y.saturating_add_signed(sy as i16),
            box_w,
            box_h,
        );

        render_ring(frame, c.votes, screen_rect, area);
    }
}
