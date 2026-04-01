use crate::renderer::Renderer;
use crate::scene::OverlayKind;
use crossterm::style::Color;

pub fn format_time(h: u32, m: u32, s: u32) -> String {
    format!("{:02}:{:02}:{:02}", h, m, s)
}

pub fn render(renderer: &mut Renderer, kind: OverlayKind) {
    match kind {
        OverlayKind::None          => {}
        OverlayKind::Clock         => render_clock(renderer),
        OverlayKind::Quote(text)   => render_quote(renderer, text),
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
