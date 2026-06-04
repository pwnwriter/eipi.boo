#[derive(Clone, Debug)]
pub struct Confession {
    pub id: i64,
    pub text: String,
    pub x: i64,
    pub y: i64,
    pub votes: i64,
    #[allow(dead_code)]
    pub author_fingerprint: String,
    #[allow(dead_code)]
    pub created_at: String,
}

pub const MAX_LENGTH: usize = 280;
pub const BOX_WIDTH: u16 = 42;
pub const BOX_INNER_WIDTH: usize = (BOX_WIDTH - 4) as usize; // borders + padding

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .replace('0', "o")
        .replace('1', "i")
        .replace('3', "e")
        .replace('4', "a")
        .replace('5', "s")
        .replace('7', "t")
        .replace('@', "a")
        .replace('$', "s")
        .replace('!', "i")
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

pub fn is_allowed(text: &str) -> bool {
    use crate::blocked_items::BLOCKED_WORDS;

    let normalized = normalize(text);
    let lower = text.to_lowercase();
    !BLOCKED_WORDS
        .iter()
        .any(|w| normalized.contains(w) || lower.contains(w))
}

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if word.len() > max_width {
            // Break long words
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            let mut remaining = word;
            while remaining.len() > max_width {
                let (chunk, rest) = remaining.split_at(max_width);
                lines.push(chunk.to_string());
                remaining = rest;
            }
            if !remaining.is_empty() {
                current_line = remaining.to_string();
            }
        } else if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

pub fn confession_height(text: &str) -> u16 {
    let lines = wrap_text(text, BOX_INNER_WIDTH);
    lines.len() as u16 + 2 // +2 for top/bottom border
}
