use crate::renderer::Renderer;
use crate::scene::{AmbientFlags, SceneConfig, SkyVariant};
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

    pub fn update(&mut self, width: u16, height: u16, rng: &mut impl Rng) {
        self.y += 0.7 + rng.random::<f32>() * 0.3; // 0.7–1.0, slight variance
        self.x -= 0.1;
        if self.y >= height as f32 {
            self.y = -rng.random::<f32>() * 4.0; // stagger re-entry
            self.x = rng.random::<f32>() * width as f32;
        }
        if self.x < 0.0 {
            self.x = width as f32 - 1.0;
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

// ── Airplane ──────────────────────────────────────────────────────────────

const AIRPLANE_ART: &[&str] = &[
    r"  __",
    r" \  \     _ _",
    r"  \**\ ___\/ \",
    r"X*#####*+^^\_\",
    r"  o/\  \",
    r"     \__\",
];

pub struct Airplane {
    x: f32,
    y: u16,
    speed: f32,
    /// Countdown in frames before the next airplane spawns. `None` = one is active.
    cooldown: u32,
}

impl Airplane {
    fn new_idle(rng: &mut impl Rng) -> Self {
        // Wait 300–900 frames (~20–60 s at 15 FPS) before first appearance
        Airplane {
            x: 0.0,
            y: 0,
            speed: 0.0,
            cooldown: 300 + (rng.random::<u32>() % 600),
        }
    }

    fn spawn(&mut self, width: u16, rng: &mut impl Rng) {
        self.y = 1 + (rng.random::<u16>() % 3); // rows 1–3
        self.speed = 0.3 + rng.random::<f32>() * 0.4; // 0.3–0.7 chars/frame
        // Start just off the right edge
        self.x = width as f32 + 2.0;
        self.cooldown = 0;
    }

    fn is_active(&self) -> bool {
        self.cooldown == 0
    }

    fn update(&mut self, width: u16, rng: &mut impl Rng) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            if self.cooldown == 0 {
                self.spawn(width, rng);
            }
            return;
        }
        // Fly from right to left
        self.x -= self.speed;
        let art_width = AIRPLANE_ART.iter().map(|r| r.len()).max().unwrap_or(0) as f32;
        if self.x < -art_width {
            // Off screen — go idle again
            self.cooldown = 300 + (rng.random::<u32>() % 600);
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        if !self.is_active() { return; }
        let sx = self.x.round() as i32;
        for (i, row) in AIRPLANE_ART.iter().enumerate() {
            let y = self.y + i as u16;
            for (j, ch) in row.chars().enumerate() {
                let col = sx + j as i32;
                if ch != ' ' && col >= 0 && (col as u16) < renderer.width() {
                    renderer.put(col as u16, y, ch, Color::DarkGrey);
                }
            }
        }
    }
}

// ── Shooting star ─────────────────────────────────────────────────────────

pub struct ShootingStar {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: u16,     // frames remaining
    cooldown: u32, // frames until next spawn
}

impl ShootingStar {
    fn new_idle(rng: &mut impl Rng) -> Self {
        ShootingStar {
            x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, life: 0,
            cooldown: 200 + (rng.random::<u32>() % 400), // 13–40 s at 15 FPS
        }
    }

    fn spawn(&mut self, width: u16, rng: &mut impl Rng) {
        // Start in the upper quarter of the sky, random x
        self.x = rng.random::<f32>() * width as f32;
        self.y = rng.random::<f32>() * 4.0 + 1.0; // rows 1–5
        // Diagonal streak: fast horizontal, moderate vertical
        self.vx = 1.5 + rng.random::<f32>() * 1.5; // 1.5–3.0
        if rng.random::<bool>() { self.vx = -self.vx; } // random direction
        self.vy = 0.3 + rng.random::<f32>() * 0.4;  // 0.3–0.7 downward
        self.life = 8 + (rng.random::<u16>() % 10);  // 8–17 frames
        self.cooldown = 0;
    }

    fn is_active(&self) -> bool {
        self.cooldown == 0 && self.life > 0
    }

    fn update(&mut self, width: u16, rng: &mut impl Rng) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            if self.cooldown == 0 {
                self.spawn(width, rng);
            }
            return;
        }
        if self.life == 0 { return; }
        self.x += self.vx;
        self.y += self.vy;
        self.life -= 1;
        if self.life == 0 {
            self.cooldown = 200 + (rng.random::<u32>() % 400);
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        if !self.is_active() { return; }
        // Draw a short tail behind the head
        let tail_len = 3u8;
        for i in 0..=tail_len {
            let tx = self.x - self.vx * i as f32 * 0.5;
            let ty = self.y - self.vy * i as f32 * 0.5;
            let col = tx.round() as i32;
            let row = ty.round() as i32;
            if col < 0 || row < 0 { continue; }
            let (col, row) = (col as u16, row as u16);
            if col >= renderer.width() { continue; }
            let (ch, color) = match i {
                0 => ('*', Color::White),
                1 => ('-', Color::Rgb { r: 200, g: 200, b: 255 }),
                _ => ('.', Color::DarkGrey),
            };
            renderer.put(col, row, ch, color);
        }
    }
}

// ── Sleeping cat ──────────────────────────────────────────────────────────

// Two frames: inhale / exhale — the belly rises and falls.
const CAT_INHALE: &[&str] = &[
    r" /\_/\ ",
    r"( -.- )",
    r" (> <)_/",
    r"  ~~~ ",
];
const CAT_EXHALE: &[&str] = &[
    r" /\_/\ ",
    r"( -.- )",
    r" ()_()_/",
    r"  ~~~ ",
];

// "Zzz" bubble offsets from top-left of cat art
const ZZZ_DX: i32 = 8;
const ZZZ_DY: i32 = -1;

pub struct SleepingCat {
    breath_phase: f32, // 0..TAU, controls inhale/exhale
    snore_frame: u64,  // tracks Zzz animation
}

impl SleepingCat {
    fn new() -> Self {
        SleepingCat { breath_phase: 0.0, snore_frame: 0 }
    }

    fn update(&mut self) {
        self.breath_phase += 0.08; // full cycle ≈ 79 frames ≈ 5.3 s
        if self.breath_phase > std::f32::consts::TAU {
            self.breath_phase -= std::f32::consts::TAU;
        }
        self.snore_frame = self.snore_frame.wrapping_add(1);
    }

    fn render(&self, renderer: &mut Renderer, base_x: u16, ground_y: u16) {
        let inhale = self.breath_phase.sin() > 0.0;
        let art = if inhale { CAT_INHALE } else { CAT_EXHALE };

        // Place cat on the ground to the right of the fire base
        let cx = base_x + 8;
        let base_bottom = ground_y + 3; // base_art starts at gy+1, 2 lines tall
        let cy = base_bottom.saturating_sub(art.len() as u16);
        for (i, row) in art.iter().enumerate() {
            renderer.put_str(cx, cy + i as u16, row, Color::Rgb { r: 180, g: 140, b: 100 });
        }

        // Zzz bubble — cycles through "z", "zZ", "zZz"
        let zzz_x = cx as i32 + ZZZ_DX;
        let zzz_y = cy as i32 + ZZZ_DY;
        if zzz_x >= 0 && zzz_y >= 0 {
            let phase = (self.snore_frame / 20) % 4; // change every ~1.3 s
            let txt = match phase {
                0 => "z",
                1 => "zZ",
                2 => "zZz",
                _ => "",   // brief pause
            };
            renderer.put_str(zzz_x as u16, zzz_y as u16, txt, Color::DarkGrey);
        }
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
    airplane: Airplane,
    shooting_star: ShootingStar,
    cat: Option<SleepingCat>,
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
        let airplane = Airplane::new_idle(rng);
        let shooting_star = ShootingStar::new_idle(rng);
        let cat = if flags.cat { Some(SleepingCat::new()) } else { None };
        AmbientState { fireflies, raindrops, airplane, shooting_star, cat }
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
            drop.update(width, height, rng);
        }

        // Airplane — only in outdoor scenes
        if config.sky != SkyVariant::Indoor {
            self.airplane.update(width, rng);
        }

        // Shooting star — only at night
        if config.sky == SkyVariant::Night {
            self.shooting_star.update(width, rng);
        }

        // Sleeping cat
        if config.ambient.cat {
            if self.cat.is_none() { self.cat = Some(SleepingCat::new()); }
            self.cat.as_mut().unwrap().update();
        } else {
            self.cat = None;
        }
    }

    pub fn render(&self, renderer: &mut Renderer, config: &SceneConfig, base_x: u16, ground_y: u16) {
        for ff in &self.fireflies {
            ff.render(renderer);
        }
        for drop in &self.raindrops {
            drop.render(renderer);
        }
        if config.sky != SkyVariant::Indoor {
            self.airplane.render(renderer);
        }
        if config.sky == SkyVariant::Night {
            self.shooting_star.render(renderer);
        }
        if let Some(cat) = &self.cat {
            cat.render(renderer, base_x, ground_y);
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
        drop.update(80, 40, &mut rand::rng());
        assert!(drop.y > 5.0, "raindrop y should increase (fall)");
    }
}
