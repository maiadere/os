use heapless::{HistoryBuf, Vec};

use crate::driver::videocore::framebuffer::{Color, Framebuffer};

pub static mut CONSOLE: Console = Console {
    history: HistoryBuf::new(),
};

pub struct Console {
    history: HistoryBuf<u8, 4096>,
}

const FONT_ATLAS: &[u8] = include_bytes!("../assets/ubuntu_sans_mono");
const CHAR_WIDTH: usize = 9;
const CHAR_HEIGHT: usize = 21;
const CHAR_SIZE: usize = CHAR_WIDTH * CHAR_HEIGHT;

const MAX_LINE_CHARS: usize = 128;
const MAX_LINES: usize = 32;

#[macro_export]
macro_rules! log {
    ($max:expr; $($arg:tt)*) => {{
        let console = unsafe { &mut *&raw mut crate::console::CONSOLE } ;
        match format!($max; $($arg)*) {
            Ok(b) => Ok(console.write(b.as_bytes())),
            Err(e) => Err(e),
        }
    }};
}

pub fn render(fb: &Framebuffer) {
    let console = unsafe { &mut *&raw mut crate::console::CONSOLE };
    console.render(fb);
}

impl Console {
    pub fn new() -> Self {
        Self {
            history: HistoryBuf::new(),
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.history.write(b);
        }
    }

    pub fn render(&self, fb: &Framebuffer) {
        let max_line_chars: usize = fb.virt_res.0 as usize / CHAR_WIDTH;
        let max_lines: usize = fb.virt_res.1 as usize / CHAR_HEIGHT;

        let lines = self.render_lines(max_line_chars, max_lines);

        fb.fill(Color::new(0.0, 0.0, 0.0));

        for (row, line) in lines.iter().rev().enumerate() {
            for (col, &c) in line.iter().enumerate() {
                let x = CHAR_WIDTH * col;
                let y = CHAR_HEIGHT * row;
                Self::draw_char(fb, x, y, c);
            }
        }
    }

    fn draw_char(fb: &Framebuffer, x: usize, y: usize, c: u8) {
        let char_offset = c as usize * CHAR_SIZE;
        let char_pixels = &FONT_ATLAS[char_offset..][..CHAR_SIZE];

        for (pixel_y, row) in char_pixels.chunks_exact(CHAR_WIDTH).enumerate() {
            for (pixel_x, &pixel) in row.iter().enumerate() {
                let x = (pixel_x + x) as u32;
                let y = (pixel_y + y) as u32;
                let p = pixel as f32 / 255.0;
                let color = Color::new(p, p, p);
                fb.set_pixel(x, y, color);
            }
        }
    }

    fn render_lines(
        &self,
        max_line_chars: usize,
        max_lines: usize,
    ) -> Vec<Vec<u8, MAX_LINE_CHARS>, MAX_LINES> {
        let max_line_chars = max_line_chars.min(MAX_LINE_CHARS);
        let mut lines: Vec<Vec<u8, MAX_LINE_CHARS>, MAX_LINES> = Vec::new();
        lines.push(Vec::new());

        for raw_line in self.history.split(|&b| b == b'\n').rev().take(max_lines) {
            for line in raw_line.chunks(max_line_chars).rev() {
                if lines.push(line.iter().copied().collect()).is_err() {
                    return lines;
                }
            }
        }

        lines
    }

    fn get_byte(&self, index: usize) -> u8 {
        let (left, right) = self.history.as_slices();
        if index < left.len() {
            left[index]
        } else {
            right[left.len() + index]
        }
    }
}
