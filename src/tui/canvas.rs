use rand::RngExt;

use crate::model::confession::{BOX_WIDTH, Confession, confession_height};

pub fn visible_confessions(
    confessions: &[Confession],
    cam_x: i64,
    cam_y: i64,
    width: u16,
    height: u16,
) -> Vec<usize> {
    let vx1 = cam_x;
    let vy1 = cam_y;
    let vx2 = cam_x + width as i64;
    let vy2 = cam_y + height as i64;

    confessions
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            let bw = BOX_WIDTH as i64;
            let bh = confession_height(&c.text) as i64;
            let cx2 = c.x + bw;
            let cy2 = c.y + bh;
            c.x < vx2 && cx2 > vx1 && c.y < vy2 && cy2 > vy1
        })
        .map(|(i, _)| i)
        .collect()
}

fn overlaps_any(x: i64, y: i64, w: i64, h: i64, confessions: &[Confession], padding: i64) -> bool {
    confessions.iter().any(|c| {
        let cw = BOX_WIDTH as i64;
        let ch = confession_height(&c.text) as i64;
        x < c.x + cw + padding
            && x + w + padding > c.x
            && y < c.y + ch + padding
            && y + h + padding > c.y
    })
}

pub fn random_position(confessions: &[Confession], new_text: &str) -> (i64, i64) {
    let mut rng = rand::rng();
    let count = confessions.len() as f64;
    let new_w = BOX_WIDTH as i64;
    let new_h = confession_height(new_text) as i64;
    let padding: i64 = 2;

    let cols = (count.sqrt() * 1.5).max(3.0) as i64;
    let spread_x = cols * (BOX_WIDTH as i64 + padding + 4);
    let spread_y = cols * 8; // average box height ~6 + padding

    for attempt in 0..300 {
        let extra = (attempt as i64 / 30) * (BOX_WIDTH as i64 + 4);
        let sx = spread_x + extra;
        let sy = spread_y + extra;
        let x = rng.random_range(-sx / 2..=sx / 2);
        let y = rng.random_range(-sy / 2..=sy / 2);

        if !overlaps_any(x, y, new_w, new_h, confessions, padding) {
            return (x, y);
        }
    }

    let max_y = confessions
        .iter()
        .map(|c| c.y + confession_height(&c.text) as i64)
        .max()
        .unwrap_or(0);
    let x = rng.random_range(0..spread_x);
    (x, max_y + padding + 2)
}

pub fn best_camera(confessions: &[Confession], width: u16, height: u16) -> (i64, i64) {
    if confessions.is_empty() {
        return (-(width as i64) / 2, -(height as i64) / 2);
    }

    let mut best_x: i64 = 0;
    let mut best_y: i64 = 0;
    let mut best_count = 0;

    let mut candidates: Vec<usize> = (0..confessions.len()).collect();
    candidates.sort_by(|a, b| confessions[*b].votes.cmp(&confessions[*a].votes));
    candidates.truncate(50);

    for &idx in &candidates {
        let c = &confessions[idx];
        let cam_x = c.x - width as i64 / 2 + BOX_WIDTH as i64 / 2;
        let cam_y = c.y - height as i64 / 2 + 2;
        let count = visible_confessions(confessions, cam_x, cam_y, width, height).len();
        if count > best_count {
            best_count = count;
            best_x = cam_x;
            best_y = cam_y;
        }
    }

    (best_x, best_y)
}
