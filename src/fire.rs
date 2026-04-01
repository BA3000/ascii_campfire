use crate::renderer::Renderer;
use crate::scene::SceneConfig;
use crossterm::style::Color;
use rand::{Rng, RngExt};

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl Particle {
    fn spawn(base_x: f32, base_y: f32, config: &SceneConfig, rng: &mut impl Rng) -> Self {
        let x = base_x + (rng.random::<f32>() - 0.5) * 2.0 * config.spread;
        let vy = -(config.base_speed + rng.random::<f32>() * 0.4);
        let vx = (rng.random::<f32>() - 0.5) * 0.25;
        let lifetime = 6.0 + rng.random::<f32>() * 6.0;
        Particle { x, y: base_y, vx, vy, lifetime, max_lifetime: lifetime }
    }

    pub fn tick(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.lifetime -= 1.0;
    }

    fn is_dead(&self) -> bool { self.lifetime <= 0.0 }

    fn ratio(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }
}

/// Returns the display character for a particle. `flicker` alternates between two chars.
pub fn particle_char(ratio: f32, flicker: bool) -> char {
    if ratio >= 0.7 {
        if flicker { 'W' } else { '^' }
    } else if ratio >= 0.4 {
        if flicker { '(' } else { ')' }
    } else if ratio >= 0.1 {
        if flicker { ',' } else { '\'' }
    } else {
        '.'
    }
}

pub fn particle_color(ratio: f32) -> Color {
    if ratio >= 0.7 {
        Color::Yellow
    } else if ratio >= 0.4 {
        Color::Rgb { r: 255, g: 140, b: 0 }
    } else if ratio >= 0.1 {
        Color::Rgb { r: 220, g: 60, b: 0 }
    } else {
        Color::DarkRed
    }
}

pub struct FireSystem {
    pub particles: Vec<Particle>,
}

impl FireSystem {
    pub fn new(config: &SceneConfig, base_x: f32, base_y: f32, rng: &mut impl Rng) -> Self {
        // Stagger initial lifetimes so fire doesn't all appear at once
        let particles = (0..config.particle_count)
            .map(|i| {
                let mut p = Particle::spawn(base_x, base_y, config, rng);
                p.lifetime = p.max_lifetime * (i as f32 / config.particle_count as f32);
                p
            })
            .collect();
        FireSystem { particles }
    }

    pub fn update(&mut self, config: &SceneConfig, base_x: f32, base_y: f32, rng: &mut impl Rng) {
        // Grow or shrink the list when scene switches change particle_count
        while self.particles.len() < config.particle_count {
            self.particles.push(Particle::spawn(base_x, base_y, config, rng));
        }
        while self.particles.len() > config.particle_count {
            self.particles.pop();
        }
        for p in &mut self.particles {
            p.tick();
            if p.is_dead() {
                *p = Particle::spawn(base_x, base_y, config, rng);
            }
        }
    }

    pub fn render(&self, renderer: &mut Renderer, frame: u64) {
        for p in &self.particles {
            let x = p.x.round() as u16;
            let y = p.y.round() as u16;
            // Flicker: vary char based on x position + frame so neighbours differ
            let flicker = ((p.x as u64).wrapping_add(frame)) % 2 == 0;
            renderer.put(x, y, particle_char(p.ratio(), flicker), particle_color(p.ratio()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_char_high_ratio_is_core() {
        assert!(matches!(particle_char(0.9, false), 'W' | '^'));
        assert!(matches!(particle_char(0.9, true),  'W' | '^'));
    }

    #[test]
    fn particle_char_mid_ratio_is_mid() {
        assert!(matches!(particle_char(0.5, false), '(' | ')'));
        assert!(matches!(particle_char(0.5, true),  '(' | ')'));
    }

    #[test]
    fn particle_char_low_ratio_is_tip() {
        assert!(matches!(particle_char(0.2, false), ',' | '\''));
    }

    #[test]
    fn particle_char_near_zero_is_ember() {
        assert_eq!(particle_char(0.05, false), '.');
    }

    #[test]
    fn particle_tick_decrements_lifetime() {
        let mut p = Particle { x: 5.0, y: 10.0, vx: 0.0, vy: -0.5, lifetime: 5.0, max_lifetime: 5.0 };
        p.tick();
        assert_eq!(p.lifetime, 4.0);
    }

    #[test]
    fn particle_tick_moves_position() {
        let mut p = Particle { x: 10.0, y: 10.0, vx: 0.1, vy: -0.5, lifetime: 5.0, max_lifetime: 5.0 };
        p.tick();
        assert!((p.x - 10.1).abs() < 0.001);
        assert!((p.y -  9.5).abs() < 0.001);
    }

    #[test]
    fn fire_system_respawns_dead_particles() {
        use crate::scene::SCENES;
        let config = &SCENES[0];
        let mut rng = rand::rng();
        let mut fire = FireSystem::new(config, 40.0, 20.0, &mut rng);
        for p in &mut fire.particles { p.lifetime = 0.0; }
        fire.update(config, 40.0, 20.0, &mut rng);
        assert!(fire.particles.iter().all(|p| p.lifetime > 0.0));
    }
}
