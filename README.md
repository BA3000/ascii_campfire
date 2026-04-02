# ascii_campfire

A terminal campfire animation written in Rust. Watch procedural fire particles flicker in your terminal and switch between 6 scenes with a single keypress.

```
        ✦ · · · ✦ · ✦ · · ✦
                          (C)
          ^  W  ^
         ( W ( W (
         , ' , ' ,
         . . . . .
          /\/\/\
         /________\
_______________________________
[1] Campfire          1-6:switch  q:quit
```

## Scenes

| Key | Scene | Description |
|-----|-------|-------------|
| `1` | Campfire | Night sky with stars, small fire |
| `2` | Bonfire | Large roaring fire, fireflies, seated figures |
| `3` | Fireplace | Indoor fireplace, narrow flame |
| `4` | Rainy Night | Overcast sky with falling rain |
| `5` | Clock | Current time and date displayed above the fire |
| `6` | Quote | Dawn sky with fireflies and an inspiring quote |

## Controls

| Key | Action |
|-----|--------|
| `1` – `6` | Switch scene |
| `q` | Quit |
| `Ctrl+C` | Quit |

## Build & Run

Requires Rust (stable).

```bash
git clone https://github.com/BA3000/ascii_campfire
cd ascii_campfire
cargo run --release
```

## How it works

- **Renderer** — double-buffered `Vec<Cell>`, diff-only flush to minimize terminal writes
- **Fire** — procedural particle system; each particle has position, velocity, and lifetime. Character and color are derived from the lifetime ratio (`^` / `W` → `(` `)` → `,` `'` → `.`)
- **Scenes** — parametric `SceneConfig` structs (particle count, spread, sky variant, ambient flags, overlay). Switching scenes is instant and seamless — particles continue under new parameters
- **Sky** — deterministic star field (no flicker), per-variant rendering (Night / Dawn / Overcast / Indoor)
- **Ambient** — fireflies, rain, seated ASCII figures
- **Overlay** — live clock (`chrono`) or static quote, centered above the fire
