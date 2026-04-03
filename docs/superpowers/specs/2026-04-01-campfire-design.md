# ascii_campfire — Design Spec

**Date:** 2026-04-01  
**Status:** Approved

## Overview

A terminal campfire animation written in Rust + crossterm. The user can switch between 6 predefined scenes by pressing number keys `1`–`6`. Each scene is a parametric configuration that drives a shared procedural fire particle system, sky background, ambient effects, and optional info overlay.

---

## Architecture

Single binary crate. No async (no network calls). Six source files with clear responsibilities:

```
src/
├── main.rs      — CLI entry, terminal init, run loop
├── renderer.rs  — Double-buffered cell renderer (crossterm)
├── fire.rs      — Procedural fire particle system
├── scene.rs     — SceneConfig struct + SCENES catalogue (keys 1–6)
├── sky.rs       — Sky/background layer (stars, moon, dawn, overcast, indoor)
├── ambient.rs   — Ambient effects: rain, fireflies, seated figures
└── overlay.rs   — Info overlays: clock, quote
```

---

## Renderer

Double-buffered `Vec<Cell>` (char + color), diff-only flush to stdout. On each frame:
1. `clear()` — fill buffer with default cells
2. Layers write into buffer in order (back to front)
3. `flush()` — write only cells that changed since last frame

Uses crossterm alternate screen + raw mode + hidden cursor. Minimum terminal size: 70×20.

---

## Fire Particle System (`fire.rs`)

Each `Particle`:
- `x: f32`, `y: f32` — position (rendered as `x.round() as u16`)
- `vx: f32` — horizontal drift (small, random)
- `vy: f32` — upward velocity (negative y direction)
- `lifetime: f32` — countdown from `max_lifetime` to 0
- `max_lifetime: f32`

Per tick: position += velocity, lifetime -= 1. Dead particles (lifetime ≤ 0) respawn at the fire base with fresh random params drawn from the active `SceneConfig`.

Character and color by `ratio = lifetime / max_lifetime`:

| ratio     | chars      | color            |
|-----------|------------|------------------|
| 0.7–1.0   | `W` `^`    | bright yellow    |
| 0.4–0.7   | `(` `)`    | orange           |
| 0.1–0.4   | `,` `'`    | red-orange       |
| 0.0–0.1   | `.`        | dark red         |

---

## Scene Config (`scene.rs`)

```rust
struct SceneConfig {
    name: &'static str,
    particle_count: usize,
    spread: f32,           // horizontal spawn width in chars
    base_speed: f32,       // base upward velocity
    sky: SkyVariant,       // Night | Dawn | Overcast | Indoor
    ambient: AmbientFlags, // bitflags: STARS | RAIN | FIREFLIES | FIGURES
    overlay: OverlayKind,  // None | Clock | Quote(&'static str)
}
```

### Predefined Scenes

| Key | Name        | Particles | Spread | Sky       | Ambient            | Overlay |
|-----|-------------|-----------|--------|-----------|--------------------|---------|
| `1` | Campfire    | 30        | 5      | Night     | STARS              | None    |
| `2` | Bonfire     | 80        | 12     | Night     | STARS + FIREFLIES  | None    |
| `3` | Fireplace   | 15        | 3      | Indoor    | —                  | None    |
| `4` | Rainy Night | 25        | 5      | Overcast  | RAIN               | None    |
| `5` | Clock       | 30        | 5      | Night     | STARS              | Clock   |
| `6` | Quote       | 30        | 5      | Dawn      | FIREFLIES          | Quote   |

---

## Sky Layer (`sky.rs`)

Renders into the upper portion of the terminal (rows 0 to `ground_y - 1`):

- **Night** — random `*` and `·` stars (seeded per terminal size, not re-randomized each frame), crescent moon `☽` top-right
- **Dawn** — horizontal color gradient rows from dark-blue → orange near the horizon
- **Overcast** — uniform dark grey, no stars
- **Indoor** — blank (dark background only, no sky elements)

---

## Ambient Layer (`ambient.rs`)

- **STARS** — same star field as Night sky (sky.rs handles this; ambient flag enables it)
- **RAIN** — falling `,` `/` characters, similar to weathr's raindrops system
- **FIREFLIES** — slow-moving `*` `.` particles above the ground line (from weathr's fireflies)
- **FIGURES** — static ASCII silhouettes of seated people on either side of the fire, positioned relative to `ground_y`

---

## Overlay Layer (`overlay.rs`)

Rendered last (on top of everything):

- **Clock** — current local time `HH:MM:SS` and date `Mon DD YYYY` centered above the fire
- **Quote** — a short ASCII-safe quote string centered in the upper half
- **None** — no overlay

---

## App Loop (`main.rs`)

```
30fps loop:
  fire.update(config, rng)
  ambient.update(config, rng)
  renderer.clear()
  sky::render(renderer, config)
  fire::render(renderer, &fire_state, config)
  ambient::render(renderer, &ambient_state, config)
  overlay::render(renderer, config)
  hud::render(renderer, active_key, term_size)
  renderer.flush()

  poll input (33ms timeout):
    '1'–'6' → swap active SceneConfig (particles continue, respawn under new params)
    'q' / Ctrl+C → break
    Resize → renderer.resize(w, h), reposition fire base
```

Scene switch is instant and seamless — particles are not reset, they naturally transition as dead ones respawn under the new config.

---

## HUD

Always-visible, not part of any scene:
- Bottom-left: `[1] Campfire`
- Bottom-right: `1-6: switch  q: quit`

---

## Dependencies

```toml
crossterm = "0.29"
rand = "0.10"
chrono = "0.4"   # for Clock overlay only
```

No async runtime needed.

---

## Out of Scope

- Configuration files
- Themes / color customization
- Custom quote input via CLI
- Animation speed control
