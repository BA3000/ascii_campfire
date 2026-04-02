mod ambient;
mod fire;
mod overlay;
mod renderer;
mod scene;
mod sky;

use crate::ambient::AmbientState;
use crate::fire::FireSystem;
use crate::renderer::Renderer;
use crate::scene::{SceneConfig, SCENES, scene_for_key};
use crate::sky::SkyState;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    style::Color,
};
use std::io;
use std::time::Duration;

const FRAME_DURATION: Duration = Duration::from_millis(1000 / 15);

fn ground_y(height: u16, config: &SceneConfig) -> u16 {
    (height as f32 * config.ground_y_ratio) as u16
}

fn base_x(width: u16) -> u16 {
    width / 2
}

fn render_base(renderer: &mut Renderer, config: &SceneConfig, bx: u16, gy: u16) {
    for (i, line) in config.base_art.iter().enumerate() {
        let x = bx.saturating_sub(line.len() as u16 / 2);
        renderer.put_str(x, gy + 1 + i as u16, line, Color::Rgb { r: 100, g: 70, b: 30 });
    }
}

fn render_hud(renderer: &mut Renderer, config: &SceneConfig, active_idx: usize) {
    let h = renderer.height();
    let w = renderer.width();
    if h == 0 { return; }
    let label = format!("[{}] {}", active_idx + 1, config.name);
    renderer.put_str(1, h - 1, &label, Color::DarkCyan);
    let hint = "1-6:switch  q:quit";
    let hx = w.saturating_sub(hint.len() as u16 + 1);
    renderer.put_str(hx, h - 1, hint, Color::DarkGrey);
}

fn run() -> io::Result<()> {
    let mut renderer = Renderer::new()?;
    renderer.init()?;

    let mut rng = rand::rng();
    let mut active_idx: usize = 0;
    let mut config: &SceneConfig = &SCENES[active_idx];

    let (mut w, mut h) = (renderer.width(), renderer.height());
    let mut gy = ground_y(h, config);
    let mut bx = base_x(w);

    let mut fire    = FireSystem::new(config, bx as f32, gy as f32, &mut rng);
    let mut sky     = SkyState::new(w, gy);
    let mut ambient = AmbientState::new(&config.ambient, w, gy, &mut rng);
    let mut frame: u64 = 0;

    loop {
        // Update simulation
        fire.update(config, bx as f32, gy as f32, &mut rng);
        ambient.update(config, w, h, gy, &mut rng);

        // Render layers back-to-front
        renderer.clear();
        sky.render(&mut renderer, config.sky, gy, frame);
        fire.render(&mut renderer, frame);
        render_base(&mut renderer, config, bx, gy);
        ambient.render(&mut renderer, config, bx, gy);
        overlay::render(&mut renderer, config.overlay);
        render_hud(&mut renderer, config, active_idx);
        renderer.flush()?;

        frame = frame.wrapping_add(1);

        // Poll input — blocks up to FRAME_DURATION
        if event::poll(FRAME_DURATION)? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char(c @ '1'..='6') => {
                        if let Some(new_cfg) = scene_for_key(c) {
                            active_idx = c as usize - '1' as usize;
                            config = new_cfg;
                            gy = ground_y(h, config);
                        }
                    }
                    _ => {}
                },
                Event::Resize(nw, nh) => {
                    renderer.resize(nw, nh)?;
                    w = nw; h = nh;
                    bx = base_x(w);
                    gy = ground_y(h, config);
                    sky.resize(w, gy);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
