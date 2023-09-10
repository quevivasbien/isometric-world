mod utils;
mod triangles;
mod scene;
mod terrain;

use scene::{Scene, Camera};
use terrain::perlin_layers;
use wasm_bindgen::{prelude::*, Clamped};

use crate::utils::set_panic_hook;

type Vertex = [f32; 2];

type Pos2 = [i32; 2];
type Pos3 = [i32; 3];

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

pub struct Matrix<T: Copy> { 
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T: Copy> Matrix<T> {
    pub fn new(data: Vec<T>, rows: usize, cols: usize) -> Self {
        assert_eq!(data.len(), rows * cols);
        Matrix { data, rows, cols }
    }
    pub fn nrows(&self) -> usize {
        self.rows
    }
    pub fn ncols(&self) -> usize {
        self.cols
    }
    pub fn get(&self, i: usize, j: usize) -> T {
        let idx = i * self.cols + j;
        self.data[idx]
    }
    pub fn set(&mut self, i: usize, j: usize, value: T) {
        let idx = i * self.cols + j;
        self.data[idx] = value;
    }
}

pub struct Canvas {
    pub data: Vec<u8>,
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
}

#[wasm_bindgen]
pub struct StateManager {
    scene: Scene,
    camera: Camera,
    canvas: Canvas,
}

#[wasm_bindgen]
impl StateManager {
    pub fn new(
        height: usize, width: usize, perlin_periods: Vec<usize>, perlin_amplitudes: Vec<f32>,
        pixel_height: usize, pixel_width: usize, scale: f32,
    ) -> Self {
        set_panic_hook();
        let max_amp = perlin_amplitudes.clone().into_iter().reduce(|acc, x| acc.max(x)).unwrap();
        let heightmap = perlin_layers(height, width, perlin_periods, perlin_amplitudes);
        let scene = Scene::from_heightmap(heightmap, -(max_amp as i32));
        let camera = Camera::new([0., 0.], pixel_height, pixel_width, scale);
        let canvas = Canvas::new(pixel_height, pixel_width);
        Self {
            scene, camera, canvas
        }
    }

    pub fn draw(&mut self) {
        self.canvas = self.scene.draw(&self.camera);
    }

    pub fn get_canvas(&self) -> Clamped<Vec<u8>> {
        Clamped(self.canvas.data.clone())
    }

    pub fn shift(&mut self, dx: f32, dy: f32) {
        self.camera.origin = [self.camera.origin[0] + dx, self.camera.origin[1] + dy];
    }
}
