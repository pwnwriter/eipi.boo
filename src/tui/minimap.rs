use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Clear};

use crate::helper::consts;
use crate::model::confession::{self, BOX_WIDTH, Confession, total_reactions};

use super::theme::Theme;

// inner (plot) dimensions; the bordered box is 2 cells larger on each axis
const INNER_W: u16 = 22;
const INNER_H: u16 = 9;
const MARGIN: u16 = 1;

/// Render a corner overlay minimap of the whole confession cloud with a box
/// outline marking the current viewport. `canvas_area` is where the 2D canvas
/// is drawn; `cam_*`/`view_*` describe the on-screen camera window.
pub fn render(
    frame: &mut Frame,
    confessions: &[Confession],
    cam_x: i64,
    cam_y: i64,
    view_w: u16,
    view_h: u16,
    canvas_area: Rect,
    theme: &Theme,
) {
    let outer_w = INNER_W + 2;
    let outer_h = INNER_H + 2;

    // need room for the box plus a margin, otherwise skip silently
    if canvas_area.width < outer_w + MARGIN || canvas_area.height < outer_h + MARGIN {
        return;
    }

    let outer = Rect::new(
        canvas_area.x + canvas_area.width - outer_w - MARGIN,
        canvas_area.y + MARGIN,
        outer_w,
        outer_h,
    );

    frame.render_widget(Clear, outer);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_dim))
        .title(Span::styled(" map ", Style::default().fg(theme.text_dim)));
    let inner = block.inner(outer);
    frame.render_widget(block, outer);

    // world bounds cover every confession *and* the current viewport, so the
    // view box is always visible even when the camera roams past the cloud.
    let (mut min_x, mut min_y) = (cam_x, cam_y);
    let (mut max_x, mut max_y) = (cam_x + view_w as i64, cam_y + view_h as i64);
    for c in confessions {
        let h = confession::confession_height(&c.text) as i64;
        min_x = min_x.min(c.x);
        min_y = min_y.min(c.y);
        max_x = max_x.max(c.x + BOX_WIDTH as i64);
        max_y = max_y.max(c.y + h);
    }

    let span_x = (max_x - min_x).max(1) as f64;
    let span_y = (max_y - min_y).max(1) as f64;
    let last_col = (INNER_W - 1) as f64;
    let last_row = (INNER_H - 1) as f64;

    let to_cell = |wx: i64, wy: i64| -> (u16, u16) {
        let col = ((wx - min_x) as f64 / span_x * last_col).round();
        let row = ((wy - min_y) as f64 / span_y * last_row).round();
        (
            col.clamp(0.0, last_col) as u16,
            row.clamp(0.0, last_row) as u16,
        )
    };

    let buf = frame.buffer_mut();

    // plot each confession as a dot, brightness by reaction tier
    for c in confessions {
        let h = confession::confession_height(&c.text) as i64;
        let (col, row) = to_cell(c.x + BOX_WIDTH as i64 / 2, c.y + h / 2);
        let total = total_reactions(c);
        let (ch, color) = if total > consts::VOTES_MAGENTA {
            ('●', theme.glow_high)
        } else if total > consts::VOTES_CYAN {
            ('∘', theme.glow_mid)
        } else {
            ('·', theme.text_dim)
        };
        let cell = &mut buf[(inner.x + col, inner.y + row)];
        // don't let a plain dot overwrite an already-glowing cell
        if cell.symbol() != "●" {
            cell.set_char(ch);
            cell.set_style(Style::default().fg(color));
        }
    }

    // viewport rectangle outline on top of the dots
    let (c0, r0) = to_cell(cam_x, cam_y);
    let (c1, r1) = to_cell(cam_x + view_w as i64, cam_y + view_h as i64);
    let (c0, c1) = (c0.min(c1), c0.max(c1));
    let (r0, r1) = (r0.min(r1), r0.max(r1));

    let vp = Style::default().fg(theme.accent);
    let mut mark = |col: u16, row: u16, ch: char| {
        let cell = &mut buf[(inner.x + col, inner.y + row)];
        cell.set_char(ch);
        cell.set_style(vp);
    };

    if c0 == c1 && r0 == r1 {
        mark(c0, r0, '▪');
        return;
    }

    for col in c0..=c1 {
        mark(col, r0, '─');
        mark(col, r1, '─');
    }
    for row in r0..=r1 {
        mark(c0, row, '│');
        mark(c1, row, '│');
    }
    mark(c0, r0, '┌');
    mark(c1, r0, '┐');
    mark(c0, r1, '└');
    mark(c1, r1, '┘');
}
