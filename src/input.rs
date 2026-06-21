#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Browse,
    Compose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEvent {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Tab,
    Backspace,
    Escape,
    Char(char),
    MouseClick(u16, u16),
}

// SGR mouse: \x1b[<button;col;rowM (press) or \x1b[<button;col;rowm (release)
fn parse_sgr_mouse(data: &[u8]) -> Option<(KeyEvent, usize)> {
    let end = data.iter().position(|&b| b == b'M' || b == b'm')?;
    let is_press = data[end] == b'M';
    let params = std::str::from_utf8(&data[3..end]).ok()?;
    let mut parts = params.split(';');
    let button: u16 = parts.next()?.parse().ok()?;
    let col: u16 = parts.next()?.parse().ok()?;
    let row: u16 = parts.next()?.parse().ok()?;

    if is_press && button == 0 {
        Some((KeyEvent::MouseClick(col.saturating_sub(1), row.saturating_sub(1)), end + 1))
    } else {
        Some((KeyEvent::Char('\0'), end + 1))
    }
}

pub fn parse(data: &[u8]) -> Vec<KeyEvent> {
    let mut events = Vec::new();
    let mut i = 0;

    while i < data.len() {
        match data[i] {
            0x1b => {
                if i + 2 < data.len() && data[i + 1] == b'[' {
                    if data[i + 2] == b'<' {
                        if let Some((event, end)) = parse_sgr_mouse(&data[i..]) {
                            events.push(event);
                            i += end;
                        } else {
                            i += 3;
                        }
                    } else {
                        match data[i + 2] {
                            b'A' => events.push(KeyEvent::Up),
                            b'B' => events.push(KeyEvent::Down),
                            b'C' => events.push(KeyEvent::Right),
                            b'D' => events.push(KeyEvent::Left),
                            b'Z' => events.push(KeyEvent::Tab),
                            _ => {}
                        }
                        i += 3;
                    }
                } else {
                    events.push(KeyEvent::Escape);
                    i += 1;
                }
            }
            0x0d | 0x0a => {
                events.push(KeyEvent::Enter);
                i += 1;
            }
            0x09 => {
                events.push(KeyEvent::Tab);
                i += 1;
            }
            0x7f | 0x08 => {
                events.push(KeyEvent::Backspace);
                i += 1;
            }
            0x03 => {
                events.push(KeyEvent::Char('q'));
                i += 1;
            }
            b if b >= 0x20 && b < 0x7f => {
                events.push(KeyEvent::Char(b as char));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    events
}
