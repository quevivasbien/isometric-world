mod utils;
mod triangles;
mod scene;

use scene::Scene;
use wasm_bindgen::prelude::*;

type Vertex = [f32; 2];
type Pos2 = [i32; 2];
type Pos3 = [i32; 3];

const THETA: f32 = std::f32::consts::FRAC_PI_6;

#[derive(Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn scaled(&self, c: f32) -> Color {
        Color {
            r: (((self.r as f32) * c) as u8).min(255).max(0),
            g: (((self.g as f32) * c) as u8).min(255).max(0),
            b: (((self.b as f32) * c) as u8).min(255).max(0),
        }
    }
}

pub struct Block {
    pub origin: Pos3,
    pub color: Color,
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

    fn proj(&self, v: Pos2) -> Vertex {
        [
            self.0 * (v[0] as f32) + self.1 * (v[1] as f32),
            self.2 * (v[0] as f32) + self.3 * (v[1] as f32),
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


#[wasm_bindgen]
pub fn render_test(h: usize, w: usize) -> Vec<u8> {
    utils::set_panic_hook();
    let mut canvas = Canvas::new(h, w);
    let blocks = vec![
        Block { origin: [0, 0, 0], color: Color { r: 255, g: 0, b: 0 } },
        Block { origin: [0, 1, 0], color: Color { r: 255, g: 0, b: 0 } },
        Block { origin: [0, 1, 1], color: Color { r: 255, g: 0, b: 124 } },
    ];
    let scene = Scene { blocks };
    let proj_matrix = ProjectionMatrix::new(32.);
    scene.draw(&proj_matrix, &[100., 100.], &mut canvas);
    
    canvas.get_data().clone()
}
