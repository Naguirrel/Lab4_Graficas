use image::{ImageBuffer, Rgb};
use std::path::Path;

pub const BACKGROUND: u32 = 0x00131C2B;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![BACKGROUND; width * height],
            depth: vec![f32::NEG_INFINITY; width * height],
        }
    }

    pub fn point(&mut self, x: i32, y: i32, depth: f32, color: u32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = y as usize * self.width + x as usize;
        if depth > self.depth[index] {
            self.depth[index] = depth;
            self.pixels[index] = color;
        }
    }

    pub fn edge_point(&mut self, x: i32, y: i32, depth: f32, color: u32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = y as usize * self.width + x as usize;
        if depth >= self.depth[index] - 0.025 {
            self.pixels[index] = color;
        }
    }

    pub fn save_png(&self, path: &Path) -> Result<(), image::ImageError> {
        let image = ImageBuffer::from_fn(self.width as u32, self.height as u32, |x, y| {
            let color = self.pixels[y as usize * self.width + x as usize];
            Rgb([(color >> 16) as u8, (color >> 8) as u8, color as u8])
        });
        image.save(path)
    }
}
