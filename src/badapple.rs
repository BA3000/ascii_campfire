use std::fs;
use std::path::PathBuf;

/// One frame of the Bad Apple animation — a list of pre-split lines.
pub struct BadApplePlayer {
    frames: Vec<Vec<String>>,
    current: usize,
}

impl BadApplePlayer {
    /// Load frames from `assets/badapple.txt`.
    ///
    /// File format: frames separated by a line containing only `---`.
    /// Each frame is a block of text lines (ASCII art).
    pub fn load() -> Option<Self> {
        let path = Self::asset_path();
        let data = fs::read_to_string(&path).ok()?;
        let frames = Self::parse_frames(&data);
        if frames.is_empty() {
            return None;
        }
        Some(BadApplePlayer { frames, current: 0 })
    }

    fn asset_path() -> PathBuf {
        // Try next to the executable first, then fall back to cwd.
        if let Ok(exe) = std::env::current_exe() {
            let dir = exe.parent().unwrap_or(exe.as_ref());
            let p = dir.join("assets").join("badapple.txt");
            if p.exists() {
                return p;
            }
        }
        PathBuf::from("assets/badapple.txt")
    }

    fn parse_frames(data: &str) -> Vec<Vec<String>> {
        let mut frames: Vec<Vec<String>> = Vec::new();
        let mut current_frame: Vec<String> = Vec::new();
        for line in data.lines() {
            if line.trim() == "---" {
                if !current_frame.is_empty() {
                    frames.push(std::mem::take(&mut current_frame));
                }
            } else {
                current_frame.push(line.to_string());
            }
        }
        if !current_frame.is_empty() {
            frames.push(current_frame);
        }
        frames
    }

    /// Advance to the next frame, looping back to the start.
    pub fn advance(&mut self) {
        self.current = (self.current + 1) % self.frames.len();
    }

    /// Return the current frame's lines.
    pub fn current_lines(&self) -> &[String] {
        &self.frames[self.current]
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_two_frames() {
        let data = "AB\nCD\n---\nEF\nGH";
        let frames = BadApplePlayer::parse_frames(data);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], vec!["AB", "CD"]);
        assert_eq!(frames[1], vec!["EF", "GH"]);
    }

    #[test]
    fn parse_trailing_frame_without_separator() {
        let data = "X\n---\nY\nZ";
        let frames = BadApplePlayer::parse_frames(data);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[1], vec!["Y", "Z"]);
    }

    #[test]
    fn advance_loops() {
        let data = "A\n---\nB\n---\nC";
        let frames = BadApplePlayer::parse_frames(data);
        let mut player = BadApplePlayer { frames, current: 0 };
        assert_eq!(player.current_lines(), &["A"]);
        player.advance();
        assert_eq!(player.current_lines(), &["B"]);
        player.advance();
        assert_eq!(player.current_lines(), &["C"]);
        player.advance();
        assert_eq!(player.current_lines(), &["A"]); // looped
    }
}
