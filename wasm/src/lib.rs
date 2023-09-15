mod utils;
mod triangles;
mod scene;
mod terrain;

use scene::{Scene, Camera};
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
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

fn float_to_u8(x: f32) -> u8 {
    (x * 256.) as u8
}

impl Color {
    pub fn scaled(&self, c: f32) -> Color {
        Color {
            r: self.r * c,
            g: self.g * c,
            b: self.b * c,
        }
    }
    pub fn bytes(&self) -> [u8; 4] {
        [float_to_u8(self.r), float_to_u8(self.g), float_to_u8(self.b), 255]
    }
}

pub struct Matrix<T: Copy> { 
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T: Copy + Default> Matrix<T> {
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
    pub fn row(&self, i: usize) -> &[T] {
        &self.data[self.cols * i..self.cols * (i + 1)]
    }
    pub fn row_mut(&mut self, i: usize) -> &mut [T] {
        &mut self.data[self.cols * i..self.cols * (i + 1)]
    }
    fn _append_below(&mut self, other: Matrix<T>) {
        assert_eq!(self.cols, other.cols);
        self.data.append(&mut other.data.clone());
        self.rows = self.rows + other.rows;
    }
    fn _append_above(&mut self, other: Matrix<T>) {
        assert_eq!(self.cols, other.cols);
        let mut new_data = other.data.clone();
        new_data.append(&mut self.data);
        self.data = new_data;
        self.rows = other.rows + self.rows;
    }
    fn _append_right(&mut self, other: Matrix<T>) {
        assert_eq!(self.rows, other.rows);
        let new_cols = self.cols + other.cols;
        let mut new_data = vec![T::default(); new_cols * self.rows];
        for i in 0..self.rows {
            new_data[i*new_cols..i*new_cols + self.cols].copy_from_slice(self.row(i));
            new_data[i*new_cols + self.cols..(i+1)*new_cols].copy_from_slice(other.row(i));
        }
        self.data = new_data;
        self.cols = new_cols;
    }
    fn _append_left(&mut self, other: Matrix<T>) {
        assert_eq!(self.rows, other.rows);
        let new_cols = self.cols + other.cols;
        let mut new_data = vec![T::default(); new_cols * self.rows];
        for i in 0..self.rows {
            new_data[i*new_cols..i*new_cols + self.cols].copy_from_slice(other.row(i));
            new_data[i*new_cols + self.cols..(i+1)*new_cols].copy_from_slice(self.row(i));
        }
        self.data = new_data;
        self.cols = new_cols;
    }
    fn displace_below(&mut self, other: Matrix<T>) {
        assert_eq!(self.cols, other.cols);
        assert!(self.rows >= other.rows);
        let shift_size = other.data.len();
        self.data.rotate_left(shift_size);
        let len = self.data.len();
        self.data[len-shift_size..].copy_from_slice(&other.data);
    }
    fn displace_above(&mut self, other: Matrix<T>) {
        assert_eq!(self.cols, other.cols);
        assert!(self.rows >= other.rows);
        let shift_size = other.data.len();
        self.data.rotate_right(shift_size);
        self.data[..shift_size].copy_from_slice(&other.data);
    }
    fn displace_right(&mut self, other: Matrix<T>) {
        assert_eq!(self.rows, other.rows);
        assert!(self.cols >= other.cols);
        let cols = self.cols;
        for i in 0..self.rows {
            let row = self.row_mut(i);
            row.rotate_left(other.cols);
            row[cols - other.cols..].copy_from_slice(other.row(i));
        }
    }
    fn displace_left(&mut self, other: Matrix<T>) {
        assert_eq!(self.rows, other.rows);
        assert!(self.cols >= other.cols);
        for i in 0..self.rows {
            let row = self.row_mut(i);
            row.rotate_right(other.cols);
            row[..other.cols].copy_from_slice(other.row(i));
        }
    }
}

pub struct Canvas(Matrix<[u8; 4]>);

impl Canvas {
    pub fn new(rows: usize, cols: usize) -> Canvas {
        Canvas(
            Matrix::new(vec![[0; 4]; rows * cols], rows, cols)
        )
    }

    pub fn set_pixel(&mut self, i: usize, j: usize, c: &Color) {
        if i >= self.0.rows || j >= self.0.cols {
            return;
        }
        self.0.set(i, j, c.bytes())
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut out = vec![0; self.0.data.len()*4];
        for (i, x) in self.0.data.iter().enumerate() {
            out[i*4..(i+1)*4].copy_from_slice(x);
        }
        out
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
        perlin_periods: Vec<usize>, perlin_amplitudes: Vec<f32>,
        pixel_height: usize, pixel_width: usize, scale: f32, seed: i32,
    ) -> Self {
        set_panic_hook();
        let max_amp = perlin_amplitudes.clone().into_iter().reduce(|acc, x| acc.max(x)).unwrap();
        let scene = Scene::new(perlin_periods, perlin_amplitudes, seed, -(max_amp as i32));
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
        Clamped(self.canvas.bytes())
    }

    pub fn shift_y(&mut self, dy: i32) {
        let new_origin = [self.camera.origin[0], self.camera.origin[1] + dy];
        if dy <= 0 {
            let temp_camera = Camera::new(new_origin, dy.abs() as usize, self.camera.width, self.camera.scale);
            let canvas_slice = self.scene.draw(&temp_camera);
            self.canvas.0.displace_above(canvas_slice.0);
        } else {
            let temp_camera = Camera::new(
                [new_origin[0], self.camera.origin[1] + self.camera.height as i32],
                dy as usize, self.camera.width, self.camera.scale
            );
            let canvas_slice = self.scene.draw(&temp_camera);
            self.canvas.0.displace_below(canvas_slice.0);
        }
        self.camera.origin = new_origin;
    }

    pub fn shift_x(&mut self, dx: i32) {
        let new_origin = [self.camera.origin[0] + dx, self.camera.origin[1]];
        if dx <= 0 {
            let temp_camera = Camera::new(new_origin, self.camera.height, dx.abs() as usize, self.camera.scale);
            let canvas_slice = self.scene.draw(&temp_camera);
            self.canvas.0.displace_left(canvas_slice.0);
        } else {
            let temp_camera = Camera::new(
                [self.camera.origin[0] + self.camera.width as i32, new_origin[1]],
                self.camera.height, dx as usize, self.camera.scale,
            );
            let canvas_slice = self.scene.draw(&temp_camera);
            self.canvas.0.displace_right(canvas_slice.0);
        }
        self.camera.origin = new_origin;
    }

    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.camera.origin = [self.camera.origin[0] + dx, self.camera.origin[1] + dy];
    }
}
