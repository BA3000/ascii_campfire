use crate::renderer::Renderer;
use crate::scene::{AmbientFlags, SceneConfig};
use crossterm::style::Color;
use rand::{Rng, RngExt};

// ── Fireflies ──────────────────────────────────────────────────────────────

pub struct Firefly {
    pub x: f32,
    pub y: f32,
    vx: f32,
    vy: f32,
    glow: f32,
    glow_speed: f32,
}

impl Firefly {
    pub fn new(width: u16, ground_y: u16, rng: &mut impl Rng) -> Self {
        let x = rng.random::<f32>() * width as f32;
        let min_y = ground_y.saturating_sub(12) as f32;
        let max_y = ground_y.saturating_sub(2) as f32;
        let y = min_y + rng.random::<f32>() * (max_y - min_y).max(1.0);
        Firefly {
            x, y,
            vx: (rng.random::<f32>() - 0.5) * 0.25,
            vy: (rng.random::<f32>() - 0.5) * 0.15,
            glow: rng.random::<f32>() * std::f32::consts::TAU,
            glow_speed: 0.08 + rng.random::<f32>() * 0.12,
        }
    }

    pub fn update(&mut self, width: u16, ground_y: u16, rng: &mut impl Rng) {
        self.x += self.vx;
        self.y += self.vy;
        self.glow += self.glow_speed;

        if rng.random::<f32>() < 0.03 {
            self.vx = (rng.random::<f32>() - 0.5) * 0.25;
            self.vy = (rng.random::<f32>() - 0.5) * 0.15;
        }

        // Wrap horizontally
        if self.x < 0.0 { self.x = width as f32; }
        if self.x > width as f32 { self.x = 0.0; }

        // Bounce vertically within band above ground
        let min_y = ground_y.saturating_sub(12) as f32;
        let max_y = ground_y.saturating_sub(2) as f32;
        if self.y < min_y { self.y = min_y; self.vy =  self.vy.abs(); }
        if self.y > max_y { self.y = max_y; self.vy = -self.vy.abs(); }
    }

    fn render(&self, renderer: &mut Renderer) {
        let b = (self.glow.sin() + 1.0) / 2.0;
        let (ch, color) = if b > 0.7 {
            ('*', Color::Yellow)
        } else if b > 0.3 {
            ('+', Color::Rgb { r: 180, g: 180, b: 0 })
        } else {
            ('.', Color::DarkYellow)
        };
        renderer.put(self.x.round() as u16, self.y.round() as u16, ch, color);
    }
}

// ── Rain ───────────────────────────────────────────────────────────────────

pub struct RainDrop {
    pub x: f32,
    pub y: f32,
}

impl RainDrop {
    fn new(width: u16, height: u16, rng: &mut impl Rng) -> Self {
        RainDrop {
            x: rng.random::<f32>() * width as f32,
            y: rng.random::<f32>() * height as f32,
        }
    }

    pub fn update(&mut self, width: u16, height: u16) {
        self.y += 0.8;
        self.x -= 0.1;
        if self.y >= height as f32 {
            self.y = 0.0;
            self.x = (self.x + 11.0) % width as f32;
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        let ch = if (self.y as u32) % 2 == 0 { ',' } else { '|' };
        renderer.put(
            self.x.round() as u16,
            self.y.round() as u16,
            ch,
            Color::Rgb { r: 100, g: 150, b: 220 },
        );
    }
}

// ── Seated figures (static ASCII silhouettes) ──────────────────────────────

const FIGURE_L: &[&str] = &[" o ", "/|\\", "/ \\"];
const FIGURE_R: &[&str] = &[" o ", "/|\\", "\\ /"];

fn render_figures(renderer: &mut Renderer, base_x: u16, ground_y: u16) {
    let w = renderer.width();
    let offset = 10u16;

    let lx = base_x.saturating_sub(offset);
    let gy = ground_y.saturating_sub(2);
    for (i, row) in FIGURE_L.iter().enumerate() {
        renderer.put_str(lx, gy + i as u16, row, Color::DarkGrey);
    }

    let rx = (base_x + offset).min(w.saturating_sub(4));
    for (i, row) in FIGURE_R.iter().enumerate() {
        renderer.put_str(rx, gy + i as u16, row, Color::DarkGrey);
    }
}

// ── AmbientState ──────────────────────────────────────────────────────────

pub struct AmbientState {
    pub fireflies: Vec<Firefly>,
    pub raindrops: Vec<RainDrop>,
}

impl AmbientState {
    pub fn new(flags: &AmbientFlags, width: u16, ground_y: u16, rng: &mut impl Rng) -> Self {
        let fireflies = if flags.fireflies {
            (0..20).map(|_| Firefly::new(width, ground_y, rng)).collect()
        } else {
            vec![]
        };
        let raindrops = if flags.rain {
            (0..60).map(|_| RainDrop::new(width, ground_y, rng)).collect()
        } else {
            vec![]
        };
        AmbientState { fireflies, raindrops }
    }

    pub fn update(
        &mut self,
        config: &SceneConfig,
        width: u16,
        height: u16,
        ground_y: u16,
        rng: &mut impl Rng,
    ) {
        // Sync firefly list to scene flag
        if config.ambient.fireflies && self.fireflies.is_empty() {
            self.fireflies = (0..20).map(|_| Firefly::new(width, ground_y, rng)).collect();
        } else if !config.ambient.fireflies {
            self.fireflies.clear();
        }
        for ff in &mut self.fireflies {
            ff.update(width, ground_y, rng);
        }

        // Sync raindrop list to scene flag
        if config.ambient.rain && self.raindrops.is_empty() {
            self.raindrops = (0..60).map(|_| RainDrop::new(width, ground_y, rng)).collect();
        } else if !config.ambient.rain {
            self.raindrops.clear();
        }
        for drop in &mut self.raindrops {
            drop.update(width, height);
        }
    }

    pub fn render(&self, renderer: &mut Renderer, config: &SceneConfig, base_x: u16, ground_y: u16) {
        for ff in &self.fireflies {
            ff.render(renderer);
        }
        for drop in &self.raindrops {
            drop.render(renderer);
        }
        if config.ambient.figures {
            render_figures(renderer, base_x, ground_y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firefly_update_does_not_panic() {
        let mut ff = Firefly::new(80, 20, &mut rand::rng());
        ff.update(80, 20, &mut rand::rng());
    }

    #[test]
    fn raindrop_falls_downward() {
        let mut drop = RainDrop { x: 10.0, y: 5.0 };
        drop.update(80, 40);
        assert!(drop.y > 5.0, "raindrop y should increase (fall)");
    }
}
