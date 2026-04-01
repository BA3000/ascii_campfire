#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SkyVariant {
    Night,
    Dawn,
    Overcast,
    Indoor,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AmbientFlags {
    pub stars: bool,
    pub rain: bool,
    pub fireflies: bool,
    pub figures: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OverlayKind {
    None,
    Clock,
    Quote(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct SceneConfig {
    pub name: &'static str,
    pub particle_count: usize,
    pub spread: f32,          // horizontal spawn half-width in chars
    pub base_speed: f32,      // upward velocity of new particles
    pub ground_y_ratio: f32,  // multiply by term_height to get ground row
    pub sky: SkyVariant,
    pub ambient: AmbientFlags,
    pub overlay: OverlayKind,
    pub base_art: &'static [&'static str], // rendered below fire base
}

pub const SCENES: &[SceneConfig] = &[
    // 1 — Campfire
    SceneConfig {
        name: "Campfire",
        particle_count: 30,
        spread: 3.0,
        base_speed: 0.6,
        ground_y_ratio: 0.72,
        sky: SkyVariant::Night,
        ambient: AmbientFlags { stars: true, rain: false, fireflies: false, figures: false },
        overlay: OverlayKind::None,
        base_art: &[r" /\/\/\ ", r"/________\"],
    },
    // 2 — Bonfire
    SceneConfig {
        name: "Bonfire",
        particle_count: 80,
        spread: 6.0,
        base_speed: 0.9,
        ground_y_ratio: 0.72,
        sky: SkyVariant::Night,
        ambient: AmbientFlags { stars: true, rain: false, fireflies: true, figures: true },
        overlay: OverlayKind::None,
        base_art: &[r"  /\/\/\/\  ", r" /\/\/\/\/\ ", r"/____________\"],
    },
    // 3 — Fireplace
    SceneConfig {
        name: "Fireplace",
        particle_count: 15,
        spread: 1.5,
        base_speed: 0.5,
        ground_y_ratio: 0.68,
        sky: SkyVariant::Indoor,
        ambient: AmbientFlags { stars: false, rain: false, fireflies: false, figures: false },
        overlay: OverlayKind::None,
        base_art: &[r"|        |", r"|________|", r" \______/ "],
    },
    // 4 — Rainy Night
    SceneConfig {
        name: "Rainy Night",
        particle_count: 25,
        spread: 3.0,
        base_speed: 0.5,
        ground_y_ratio: 0.72,
        sky: SkyVariant::Overcast,
        ambient: AmbientFlags { stars: false, rain: true, fireflies: false, figures: false },
        overlay: OverlayKind::None,
        base_art: &[r" /\/\/\ ", r"/________\"],
    },
    // 5 — Clock
    SceneConfig {
        name: "Clock",
        particle_count: 30,
        spread: 3.0,
        base_speed: 0.6,
        ground_y_ratio: 0.72,
        sky: SkyVariant::Night,
        ambient: AmbientFlags { stars: true, rain: false, fireflies: false, figures: false },
        overlay: OverlayKind::Clock,
        base_art: &[r" /\/\/\ ", r"/________\"],
    },
    // 6 — Quote
    SceneConfig {
        name: "Quote",
        particle_count: 30,
        spread: 3.0,
        base_speed: 0.6,
        ground_y_ratio: 0.72,
        sky: SkyVariant::Dawn,
        ambient: AmbientFlags { stars: false, rain: false, fireflies: true, figures: false },
        overlay: OverlayKind::Quote("Not all those who wander are lost."),
        base_art: &[r" /\/\/\ ", r"/________\"],
    },
];

pub fn scene_for_key(key: char) -> Option<&'static SceneConfig> {
    let idx = key.to_digit(10)? as usize;
    if idx >= 1 && idx <= SCENES.len() {
        Some(&SCENES[idx - 1])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_scenes_defined() {
        assert_eq!(SCENES.len(), 6);
    }

    #[test]
    fn key_1_returns_campfire() {
        let s = scene_for_key('1').unwrap();
        assert_eq!(s.name, "Campfire");
    }

    #[test]
    fn key_6_returns_quote() {
        let s = scene_for_key('6').unwrap();
        assert_eq!(s.name, "Quote");
    }

    #[test]
    fn invalid_key_returns_none() {
        assert!(scene_for_key('0').is_none());
        assert!(scene_for_key('7').is_none());
    }
}
