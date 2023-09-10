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
fn to_vertex(p: Pos2) -> Vertex {
    [p[0] as f32, p[1] as f32]
} 

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

    pub fn row(&self, i: usize) -> &[u8] {
        &self.data[4 * self.cols * i..4 * self.cols * (i + 1)]
    }

    pub fn row_mut(&mut self, i: usize) -> &mut [u8] {
        &mut self.data[4 * self.cols * i..4 * self.cols * (i + 1)]
    }

    pub fn size(&self) -> usize {
        self.rows * self.cols * 4
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
        let camera = Camera::new([0, 0], pixel_height, pixel_width, scale);
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

    pub fn shift_y(&mut self, dy: i32) {
        let new_origin = [self.camera.origin[0], self.camera.origin[1] + dy];
        if dy <= 0 {
            let temp_camera = Camera::new(new_origin, dy.abs() as usize, self.camera.width, self.camera.scale);
            let canvas_slice = self.scene.draw(&temp_camera);
            let shift_size = canvas_slice.size();
            self.canvas.data.rotate_right(shift_size);
            self.canvas.data[..shift_size].copy_from_slice(&canvas_slice.data);
        } else {
            let temp_camera = Camera::new(
                [new_origin[0], self.camera.origin[1] + self.camera.height as i32],
                dy as usize, self.camera.width, self.camera.scale
            );
            let canvas_slice = self.scene.draw(&temp_camera);
            let shift_size = canvas_slice.size();
            self.canvas.data.rotate_left(shift_size);
            let start_idx = self.canvas.size() - shift_size;
            self.canvas.data[start_idx..].copy_from_slice(&canvas_slice.data);
        }
        self.camera.origin = new_origin;
    }

    pub fn shift_x(&mut self, dx: i32) {
        let new_origin = [self.camera.origin[0] + dx, self.camera.origin[1]];
        if dx <= 0 {
            let temp_camera = Camera::new(new_origin, self.camera.height, dx.abs() as usize, self.camera.scale);
            let canvas_slice = self.scene.draw(&temp_camera);
            let line_shift = canvas_slice.cols * 4;
            for i in 0..self.canvas.rows {
                let row = self.canvas.row_mut(i);
                row.rotate_right(line_shift);
                row[..line_shift].copy_from_slice(canvas_slice.row(i));
            }
        } else {
            let temp_camera = Camera::new(
                [self.camera.origin[0] + self.camera.width as i32, new_origin[1]],
                self.camera.height, dx as usize, self.camera.scale,
            );
            let canvas_slice = self.scene.draw(&temp_camera);
            let line_shift = canvas_slice.cols * 4;
            for i in 0..self.canvas.rows {
                let row = self.canvas.row_mut(i);
                row.rotate_left(line_shift);
                let start_idx = row.len() - line_shift;
                row[start_idx..].copy_from_slice(canvas_slice.row(i));
            }
        }
        self.camera.origin = new_origin;
    }

    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.camera.origin = [self.camera.origin[0] + dx, self.camera.origin[1] + dy];
    }
}
