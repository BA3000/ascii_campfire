use crossterm::{
    cursor,
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, BufWriter, IsTerminal, Stdout, Write};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cell {
    pub ch: char,
    pub color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', color: Color::Reset }
    }
}

pub struct Renderer {
    stdout: Option<BufWriter<Stdout>>,
    width: u16,
    height: u16,
    buffer: Vec<Cell>,
    prev_buffer: Vec<Cell>,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        if !io::stdout().is_terminal() {
            return Err(io::Error::new(io::ErrorKind::Other, "not a TTY"));
        }
        let (width, height) = terminal::size()?;
        let size = width as usize * height as usize;
        Ok(Renderer {
            stdout: Some(BufWriter::new(io::stdout())),
            width,
            height,
            buffer: vec![Cell::default(); size],
            prev_buffer: vec![Cell::default(); size],
        })
    }

    /// Headless constructor for tests — no stdout, no TTY required.
    pub fn new_headless(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        Renderer {
            stdout: None,
            width,
            height,
            buffer: vec![Cell::default(); size],
            prev_buffer: vec![Cell::default(); size],
        }
    }

    pub fn init(&mut self) -> io::Result<()> {
        if let Some(ref mut out) = self.stdout {
            terminal::enable_raw_mode()?;
            execute!(out, EnterAlternateScreen, cursor::Hide)?;
        }
        Ok(())
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        if let Some(ref mut out) = self.stdout {
            execute!(out, LeaveAlternateScreen, cursor::Show, ResetColor)?;
            terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn width(&self) -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }

    pub fn resize(&mut self, width: u16, height: u16) -> io::Result<()> {
        self.width = width;
        self.height = height;
        let size = width as usize * height as usize;
        self.buffer = vec![Cell::default(); size];
        self.prev_buffer = vec![Cell::default(); size];
        if let Some(ref mut out) = self.stdout {
            execute!(out, Clear(ClearType::All))?;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.buffer.fill(Cell::default());
    }

    pub fn put(&mut self, x: u16, y: u16, ch: char, color: Color) {
        if x < self.width && y < self.height {
            let idx = y as usize * self.width as usize + x as usize;
            self.buffer[idx] = Cell { ch, color };
        }
    }

    pub fn put_str(&mut self, x: u16, y: u16, s: &str, color: Color) {
        for (i, ch) in s.chars().enumerate() {
            self.put(x + i as u16, y, ch, color);
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let Some(ref mut out) = self.stdout else { return Ok(()); };
        let mut current_color = Color::Reset;
        let mut last_pos: Option<(u16, u16)> = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y as usize * self.width as usize + x as usize;
                if idx >= self.buffer.len() { continue; }
                let cell = self.buffer[idx];
                let prev = self.prev_buffer[idx];
                if cell == prev { continue; }

                let expected = last_pos.map(|(lx, ly)| (lx + 1, ly));
                if expected != Some((x, y)) {
                    queue!(out, cursor::MoveTo(x, y))?;
                }
                if cell.color != current_color {
                    queue!(out, SetForegroundColor(cell.color))?;
                    current_color = cell.color;
                }
                queue!(out, Print(cell.ch))?;
                last_pos = Some((x, y));
            }
        }

        if current_color != Color::Reset {
            queue!(out, ResetColor)?;
        }
        out.flush()?;
        self.prev_buffer.copy_from_slice(&self.buffer);
        Ok(())
    }

    #[cfg(test)]
    pub fn cell_at(&self, x: u16, y: u16) -> Cell {
        let idx = y as usize * self.width as usize + x as usize;
        self.buffer[idx]
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_fills_buffer_with_defaults() {
        let mut r = Renderer::new_headless(10, 5);
        r.put(3, 2, 'X', Color::Red);
        r.clear();
        assert_eq!(r.cell_at(3, 2), Cell::default());
    }

    #[test]
    fn put_writes_to_correct_position() {
        let mut r = Renderer::new_headless(10, 5);
        r.put(4, 2, 'A', Color::Green);
        assert_eq!(r.cell_at(4, 2).ch, 'A');
        assert_eq!(r.cell_at(4, 2).color, Color::Green);
    }

    #[test]
    fn put_ignores_out_of_bounds() {
        let mut r = Renderer::new_headless(10, 5);
        r.put(10, 0, 'X', Color::Red);
        r.put(0, 5, 'X', Color::Red);
    }

    #[test]
    fn put_str_writes_characters_in_sequence() {
        let mut r = Renderer::new_headless(20, 5);
        r.put_str(3, 1, "hello", Color::White);
        assert_eq!(r.cell_at(3, 1).ch, 'h');
        assert_eq!(r.cell_at(7, 1).ch, 'o');
    }
}
