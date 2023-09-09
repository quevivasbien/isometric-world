mod utils;
mod triangles;

use wasm_bindgen::prelude::*;

type Vertex = [f32; 2];
type Vertex3D = [f32; 3];

const THETA: f32 = std::f32::consts::FRAC_PI_6;

#[derive(Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn scaled(&self, c: u8) -> Color {
        Color {
            r: u8::min(1, u8::max(0, self.r * c)),
            g: u8::min(1, u8::max(0, self.g * c)),
            b: u8::min(1, u8::max(0, self.b * c)),
        }
    }
}

pub struct Block {
    origin: Vertex3D,
    color: Color,
}

impl Block {
    pub fn draw_after(&self, other: &Block) -> bool {
        let t = self.origin;
        let o = other.origin;
        if t[2] > o[2] {
            return true
        } else if t[2] < o[2] {
            return false
        }
        if t[1] > o[1] {
            return true;
        } else if t[1] < o[1] {
            return false;
        }

        t[0] > o[0]
    }
}

pub struct ProjectionMatrix(f32, f32, f32, f32);

impl ProjectionMatrix {
    pub fn new(scale: f32) -> Self {
        Self(
            scale * f32::cos(THETA), -scale * f32::cos(THETA),
            scale * f32::sin(THETA), scale * f32::sin(THETA),
        )
    }

    fn proj(&self, v: Vertex) -> Vertex {
        [
            self.0 * v[0] + self.1 * v[1],
            self.2 * v[0] + self.3 * v[1],
        ]
    }
}

pub struct Canvas {
    data: Vec<u8>,
    rows: usize,
    cols: usize,
}

impl Canvas {
    pub fn new(rows: usize, cols: usize) -> Canvas {
        Canvas {
            data: vec![0; rows * cols * 4],
            rows,
            cols,
        }
    }

    pub fn set_pixel(&mut self, i: usize, j: usize, c: &Color) {
        if i >= self.rows || j >= self.cols {
            return;
        }
        let i0 = (i * self.cols + j) * 4;
        self.data[i0] = c.r;
        self.data[i0 + 1] = c.g;
        self.data[i0 + 2] = c.b;
        self.data[i0 + 3] = 255;
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}

pub struct Scene {
    
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(s: &str) {
    utils::set_panic_hook();
    alert(&format!("You requested: {}", s));
}
