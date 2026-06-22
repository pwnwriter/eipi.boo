use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::Frame;

use crate::confession::{self, Confession};
use crate::consts;

const SPARKLES: [char; 4] = ['✦', '✧', '·', '*'];

/// Simple hash for deterministic sparkle placement.
fn sparkle_hash(x: i64, y: i64) -> u64 {
    let mut h = (x as u64).wrapping_mul(374761393);
    h = h.wrapping_add((y as u64).wrapping_mul(668265263));
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    h
}

/// Render glow sparkles around popular confessions (votes > 10).
pub fn render(
    frame: &mut Frame,
    confessions: &[Confession],
    cam_x: i64,
    cam_y: i64,
    area: Rect,
) {
    let buf = frame.buffer_mut();

    for c in confessions {
        if c.votes <= consts::VOTES_GLOW {
            continue;
        }

        let box_h = confession::confession_height(&c.text) as i64;
        let box_w = consts::BOX_WIDTH as i64;

        // intensity: more votes = more sparkles
        let density = if c.votes > consts::VOTES_MAGENTA {
            consts::GLOW_DENSITY_HIGH
        } else {
            consts::GLOW_DENSITY_LOW
        };

        let color = if c.votes > consts::VOTES_MAGENTA {
            Color::Magenta
        } else {
            Color::Cyan
        };

        // scan a border ring around the confession box (1-2 cells out)
        for ring in 1..=2i64 {
            for wy in (c.y - ring)..=(c.y + box_h + ring - 1) {
                for wx in (c.x - ring)..=(c.x + box_w + ring - 1) {
                    // only draw on the ring edge, not inside
                    let on_edge = wx < c.x || wx >= c.x + box_w
                        || wy < c.y || wy >= c.y + box_h;
                    if !on_edge {
                        continue;
                    }

                    let h = sparkle_hash(wx, wy);
                    if !h.is_multiple_of(density) {
                        continue;
                    }

                    let sx = wx - cam_x;
                    let sy = wy - cam_y;

                    if sx < 0 || sy < 0 {
                        continue;
                    }

                    let cell_x = area.x + sx as u16;
                    let cell_y = area.y + sy as u16;

                    if cell_x >= area.x + area.width || cell_y >= area.y + area.height {
                        continue;
                    }

                    // only fill empty cells
                    let cell = &buf[(cell_x, cell_y)];
                    if cell.symbol() != " " {
                        continue;
                    }

                    let sparkle = SPARKLES[(h / 7 % SPARKLES.len() as u64) as usize];
                    // outer ring is dimmer
                    let style = if ring == 1 {
                        Style::default().fg(color)
                    } else {
                        Style::default().fg(Color::Indexed(238))
                    };

                    let cell = &mut buf[(cell_x, cell_y)];
                    cell.set_char(sparkle);
                    cell.set_style(style);
                }
            }
        }
    }
}
