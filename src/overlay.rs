use crate::badapple::BadApplePlayer;
use crate::renderer::Renderer;
use crate::scene::OverlayKind;
use crossterm::style::Color;

pub fn format_time(h: u32, m: u32, s: u32) -> String {
    format!("{:02}:{:02}:{:02}", h, m, s)
}

pub fn render(renderer: &mut Renderer, kind: OverlayKind, badapple: &mut Option<BadApplePlayer>) {
    match kind {
        OverlayKind::None          => {}
        OverlayKind::Clock         => render_clock(renderer),
        OverlayKind::Quote(text)   => render_quote(renderer, text),
        OverlayKind::BadApple      => render_badapple(renderer, badapple),
    }
}

fn render_clock(renderer: &mut Renderer) {
    use chrono::{Local, Timelike};
    let now = Local::now();
    let time_str = format_time(now.hour(), now.minute(), now.second());
    let date_str = now.format("%b %d %Y").to_string();

    let w = renderer.width();
    let h = renderer.height();
    let y = h / 5;

    let tx = w.saturating_sub(time_str.len() as u16) / 2;
    renderer.put_str(tx, y, &time_str, Color::Cyan);

    let dx = w.saturating_sub(date_str.len() as u16) / 2;
    renderer.put_str(dx, y + 1, &date_str, Color::DarkCyan);
}

fn render_badapple(renderer: &mut Renderer, player: &mut Option<BadApplePlayer>) {
    // Lazy-load on first use
    if player.is_none() {
        *player = BadApplePlayer::load();
    }
    let Some(p) = player.as_mut() else {
        // File not found — show hint
        let msg = "Place badapple.txt in assets/";
        let w = renderer.width();
        let h = renderer.height();
        let x = w.saturating_sub(msg.len() as u16) / 2;
        renderer.put_str(x, h / 2, msg, Color::DarkGrey);
        return;
    };

    let w = renderer.width() as usize;
    let h = renderer.height() as usize;
    let lines = p.current_lines();

    // Center the frame on screen
    let frame_h = lines.len();
    let y_off = if h > frame_h { (h - frame_h) / 2 } else { 0 };

    for (i, line) in lines.iter().enumerate() {
        let row = y_off + i;
        if row >= h { break; }
        let line_w = line.chars().count();
        let x_off = if w > line_w { (w - line_w) / 2 } else { 0 };
        // Write ALL characters including spaces so the diff-based flush
        // correctly clears cells that were non-space in the previous frame.
        for (j, ch) in line.chars().enumerate() {
            let col = x_off + j;
            if col >= w { break; }
            renderer.put(col as u16, row as u16, ch, Color::White);
        }
        // Fill remaining columns on this row to clear any leftover chars
        // when the terminal is wider than the frame data.
        let filled = x_off + line_w;
        for col in filled..w {
            renderer.put(col as u16, row as u16, ' ', Color::Reset);
        }
        for col in 0..x_off {
            renderer.put(col as u16, row as u16, ' ', Color::Reset);
        }
    }

    p.advance();
}

fn render_quote(renderer: &mut Renderer, text: &str) {
    let w = renderer.width();
    let h = renderer.height();
    let x = w.saturating_sub(text.len() as u16) / 2;
    let y = h / 5;
    renderer.put_str(x, y, text, Color::Rgb { r: 180, g: 160, b: 120 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_pads_single_digits() {
        assert_eq!(format_time(1, 2, 3), "01:02:03");
    }

    #[test]
    fn format_time_correct_length() {
        assert_eq!(format_time(12, 5, 9).len(), 8);
    }
}
