use crate::renderer::Renderer;
use crate::scene::SkyVariant;
use crossterm::style::Color;

/// Pre-generated star positions — stable across frames, rebuilt only on resize.
pub struct SkyState {
    stars: Vec<(u16, u16)>,
    width: u16,
    height: u16,
}

impl SkyState {
    pub fn new(width: u16, height: u16, ground_y: u16) -> Self {
        let mut stars = Vec::new();
        // Deterministic grid with gaps — no RNG so stars never flicker position
        let pattern = [true, false, false, true, false, true, false, false, false, true];
        let mut x: u16 = 3;
        let mut y: u16 = 1;
        let mut idx = 0usize;
        while y < ground_y.saturating_sub(2) {
            if pattern[idx % pattern.len()] {
                stars.push((x, y));
            }
            x = x.wrapping_add(7);
            if x >= width {
                x = (x % 5) + 1;
                y += 2;
            }
            idx += 1;
        }
        SkyState { stars, width, height }
    }

    pub fn resize(&mut self, width: u16, height: u16, ground_y: u16) {
        *self = SkyState::new(width, height, ground_y);
    }

    pub fn render(&self, renderer: &mut Renderer, variant: SkyVariant, ground_y: u16, frame: u64) {
        match variant {
            SkyVariant::Night    => self.render_night(renderer, ground_y, frame),
            SkyVariant::Dawn     => self.render_dawn(renderer, ground_y),
            SkyVariant::Overcast => self.render_overcast(renderer, ground_y),
            SkyVariant::Indoor   => {} // blank — default cells are space/Reset
        }
    }

    fn render_night(&self, renderer: &mut Renderer, ground_y: u16, frame: u64) {
        for &(x, y) in &self.stars {
            // Subtle twinkle: dim every ~30 frames, staggered by x position
            let dim = ((x as u64).wrapping_add(frame / 30)) % 3 == 0;
            let ch = if dim { '.' } else { '*' };
            renderer.put(x, y, ch, Color::White);
        }
        // Moon: top-right
        let mx = self.width.saturating_sub(6);
        renderer.put(mx,     1, '(', Color::Yellow);
        renderer.put(mx + 1, 1, 'C', Color::Yellow);
        renderer.put(mx + 2, 1, ')', Color::Yellow);
        // Ground line
        for x in 0..self.width {
            renderer.put(x, ground_y, '_', Color::DarkGrey);
        }
    }

    fn render_dawn(&self, renderer: &mut Renderer, ground_y: u16) {
        // Horizontal bands: dark-blue → blue → orange near horizon
        let bands: &[(Color, char)] = &[
            (Color::DarkBlue,                        ' '),
            (Color::DarkBlue,                        ' '),
            (Color::Blue,                            '~'),
            (Color::Rgb { r: 100, g: 60,  b: 20 },  '-'),
            (Color::Rgb { r: 180, g: 80,  b: 20 },  '-'),
            (Color::Rgb { r: 220, g: 120, b: 40 },  '~'),
        ];
        let sky_rows = ground_y as usize;
        for y in 0..ground_y {
            let band = ((y as usize) * bands.len() / sky_rows.max(1)).min(bands.len() - 1);
            let (color, ch) = bands[band];
            for x in 0..self.width {
                renderer.put(x, y, ch, color);
            }
        }
        for x in 0..self.width {
            renderer.put(x, ground_y, '_', Color::DarkGrey);
        }
    }

    fn render_overcast(&self, renderer: &mut Renderer, ground_y: u16) {
        for y in 0..ground_y {
            for x in 0..self.width {
                renderer.put(x, y, ' ', Color::DarkGrey);
            }
        }
        for x in 0..self.width {
            renderer.put(x, ground_y, '_', Color::DarkGrey);
        }
    }
}
