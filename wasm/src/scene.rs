use std::{collections::HashMap, hash::Hash, iter::StepBy};

use itertools::iproduct;
use wasm_bindgen_test::console_log;

use crate::{Vertex, Canvas, Color, triangles::Triangle, Pos2, Pos3, utils::{round_down, round_up}, to_vertex, terrain::perlin_layers};

const THETA: f32 = std::f32::consts::FRAC_PI_6;
const HEIGHTMAP_CHUNK_SIZE: i32 = 64;
const BLOCK_CHUNK_SIZE: i32 = 16;
const MAX_CHUNKMAP_SIZE: usize = 128;

struct Block {
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

#[derive(Debug)]
struct Bounds {
    x: (i32, i32),
    y: (i32, i32),
}

impl Bounds {
    pub fn contains(&self, block: &Block) -> bool {
        let [x, y, _] = block.origin;
        x >= self.x.0
            && y >= self.y.0
            && x < self.x.1
            && y < self.y.1
    }
}

// a set of blocks within some rectangle in the x, y plane
struct BlockChunk {
    bounds: Bounds,
    blocks: Vec<Block>,
}

impl BlockChunk {
    pub fn new(bounds: Bounds) -> Self {
        Self { bounds, blocks: Vec::new(), }
    }
    pub fn add(&mut self, block: Block) -> Result<(), &str> {
        if !self.bounds.contains(&block) {
            return Err("Block not within chunk bounds");
        }
        self.blocks.push(block);
        Ok(())
    }

    pub fn process_slices<'a>(&'a self, slices: &mut HashMap<SliceKey, Slice<'a>>) {
        for b in self.blocks.iter() {
            for index in 0..6 {
                let (key, val) = Slice::create(index, b);
                match slices.get(&key) {
                    Some(other) => if b.draw_after(other.parent) {
                        slices.insert(key, val);
                    },
                    None => { slices.insert(key, val); },
                }
            }
        }
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

    pub fn proj(&self, v: Vertex) -> Vertex {
        [
            self.0 * v[0] + self.1 * v[1],
            self.2 * v[0] + self.3 * v[1],
        ]
    }

    pub fn inverse(&self) -> Self {
        let det = self.0 * self.3 - self.1 * self.2;
        Self(
            self.3 / det, -self.1 / det,
            -self.2 / det, self.0 / det
        )
    }
}

pub struct Camera {
    // the point in 2d space that should be rendered at the top-right corner of the screen
    pub origin: Pos2,
    // screen dimensions in terms of pixels
    pub height: usize,
    pub width: usize,
    // pixels per block edge
    pub scale: f32,
    pub proj_matrix: ProjectionMatrix,
}

impl Camera {
    pub fn new(origin: Pos2, height: usize, width: usize, scale: f32) -> Self {
        Camera {
            origin,
            height, width,
            scale,
            proj_matrix: ProjectionMatrix::new(scale),
        }
    }

    fn in_view(&self, chunk_size: i32) -> itertools::Product<StepBy<std::ops::Range<i32>>, StepBy<std::ops::Range<i32>>> {
        // first get coordinates of screen bounds at z = 0
        let [x0, y0] = to_vertex(self.origin);
        let inv_proj = self.proj_matrix.inverse();
        let top_left = inv_proj.proj([x0, y0]);
        let top_right = inv_proj.proj([x0 + self.width as f32, y0]);
        let bottom_left = inv_proj.proj([x0, y0 + self.height as f32]);
        let bottom_right = inv_proj.proj([x0 + self.width as f32, y0 + self.height as f32]);
        // take extreme values and round to surrounding multiples of chunk_size
        let x_min = round_down(top_left[0] as i32, chunk_size) - chunk_size;
        let x_max = round_up(bottom_right[0] as i32, chunk_size) + chunk_size;
        let y_min = round_down(top_right[1] as i32, chunk_size) - chunk_size;
        let y_max = round_up(bottom_left[1] as i32, chunk_size) + chunk_size;
        iproduct!((x_min..x_max).step_by(chunk_size as usize), (y_min..y_max).step_by(chunk_size as usize))
    }

    // determines which chunks in an array of chunks are (at least partially) in view
    // assumes that hash map keys are same as chunk origins, and chunk bounds are divisible by chunk_size
    fn chunks_in_view<'a, T>(&self, chunks: &'a HashMap<Pos2, T>, chunk_size: i32) -> Vec<&'a T> {
        let mut chunks_out = Vec::<&T>::new();
        for (x, y) in self.in_view(chunk_size) {
            match chunks.get(&[x, y]) {
                Some(chunk) => chunks_out.push(chunk),
                None => (),
            }
        }
        
        chunks_out
    }
}

// represents one of 6 triangular slices that makes up the 2d isometric view of a block
struct Slice<'a> {
    pos: Pos2,
    index: u8,  // 0 through 5, starting with top-left slice and going clockwise
    parent: &'a Block,
}

#[derive(Eq, Hash, PartialEq)]
struct SliceKey(i32, i32, bool);

impl<'a> Slice<'a> {
    fn create(index: u8, parent: &'a Block) -> (SliceKey, Slice<'a>) {
        assert!(index < 6);
        let pos3 = if index == 0 || index == 5 {
            parent.origin
        } else if index == 1 {
            [parent.origin[0], parent.origin[1] - 1, parent.origin[2] - 1]
        } else if index == 4 {
            [parent.origin[0] - 1, parent.origin[1], parent.origin[2] - 1]
        } else {
            [parent.origin[0], parent.origin[1], parent.origin[2] - 1]
        };
        let pos = [pos3[0] - pos3[2], pos3[1] - pos3[2]];
        (SliceKey(pos[0], pos[1], index % 2 == 0), Self { pos, index, parent })
    }
    fn points_right(&self) -> bool {
        self.index % 2 == 0
    }
    fn color(&self) -> Color {
        if self.index == 0 || self.index == 5 {
            return self.parent.color.clone();
        } else if self.index == 1 || self.index == 2 {
            return self.parent.color.scaled(0.8);
        } else {
            return self.parent.color.scaled(0.9);
        }
    }

    fn draw(&self, proj_matrix: &ProjectionMatrix, origin: Pos2, canvas: &mut Canvas) {
        let v = if self.points_right() {
            [
                self.pos,
                [self.pos[0] + 1, self.pos[1]],
                [self.pos[0] + 1, self.pos[1] + 1],
            ]
        } else {
            [
                self.pos,
                [self.pos[0] + 1, self.pos[1] + 1],
                [self.pos[0], self.pos[1] + 1],
            ]
        };
        let mut vertices = [[0f32; 2]; 3];
        let o32 = to_vertex(origin);
        for i in 0..3 {
            let proj = proj_matrix.proj([v[i][0] as f32, v[i][1] as f32]);
            vertices[i] = [proj[0] - o32[0], proj[1] - o32[1]];
        }
        Triangle::new(vertices, self.color()).draw(canvas);
    }
}

struct SliceMap<'a>(HashMap<SliceKey, Slice<'a>>);

impl<'a> SliceMap<'a> {
    fn new() -> Self {
        Self(HashMap::new())
    }
    fn process_bchunks(&mut self, chunks: Vec<&'a BlockChunk>) {
        for c in chunks.into_iter() {
            c.process_slices(&mut self.0);
        }
    }
    fn process_hchunks(&mut self, camera: &Camera, chunks: Vec<&'a HeightmapChunk>) {
        for c in chunks.into_iter() {
            let bchunks = camera.chunks_in_view(&c.children, BLOCK_CHUNK_SIZE);
            self.process_bchunks(bchunks);
        }
    }
}

struct HeightmapChunk {
    children: HashMap<Pos2, BlockChunk>,
}

impl HeightmapChunk {

    fn add(&mut self, b: Block) {
        let chunk_x0 = round_down(b.origin[0], BLOCK_CHUNK_SIZE);
        let chunk_y0 = round_down(b.origin[1], BLOCK_CHUNK_SIZE);
        if let Some(chunk) = self.children.get_mut(&[chunk_x0, chunk_y0]) {
            chunk.add(b).unwrap();
        } else {
            let mut chunk = BlockChunk::new(
                Bounds{
                    x: (chunk_x0, chunk_x0 + BLOCK_CHUNK_SIZE),
                    y: (chunk_y0, chunk_y0 + BLOCK_CHUNK_SIZE)
                }
            );
            chunk.add(b).unwrap();
            self.children.insert([chunk_x0, chunk_y0], chunk);
        }
    }

    pub fn new(
        origin: Pos2,
        periods: &Vec<usize>, amplitudes: &Vec<f32>,
        seed: i32, min_height: i32,
    ) -> Self {
        let mut this = Self { children: HashMap::new() };
        let heightmap = perlin_layers(
            HEIGHTMAP_CHUNK_SIZE as usize, HEIGHTMAP_CHUNK_SIZE as usize,
            origin[0], origin[1],
            periods, amplitudes, seed
        );
        heightmap.data.iter().enumerate().for_each(
            |(idx, height)| {
                let (i, j) = (idx / heightmap.cols, idx % heightmap.cols);
                for z in min_height..=(*height as i32) {
                    let x = 1. / ((1 + z - min_height) as f32).powf(0.35);
                    let origin = [
                        j as i32 + origin[0],
                        i as i32 + origin[1],
                        z
                    ];
                    this.add(Block {
                        origin,
                        color: Color { r: x, g: x, b: x },
                    });
                }
            }
        );

        this
    }
}

pub struct Scene {
    chunks: HashMap<Pos2, HeightmapChunk>,
    periods: Vec<usize>,
    amplitudes: Vec<f32>,
    seed: i32, min_height: i32,
}

impl Scene {
    pub fn new(periods: Vec<usize>, amplitudes: Vec<f32>, seed: i32, min_height: i32) -> Self {
        Self { chunks: HashMap::new(), periods, amplitudes, seed, min_height }
    }

    // adds new chunks and gets rid of chunks that are no longer visible
    fn set_chunks_and_strip_old(&mut self, camera: &Camera) {
        let mut new_chunks: HashMap<Pos2, HeightmapChunk> = HashMap::new();
        for (x, y) in camera.in_view(HEIGHTMAP_CHUNK_SIZE) {
            if let Some(chunk) = self.chunks.remove(&[x, y]) {
                new_chunks.insert([x, y], chunk);
            } else {
                // console_log!("Adding chunk at ({}, {})", x, y);
                new_chunks.insert([x, y], HeightmapChunk::new(
                    [x, y],
                    &self.periods, &self.amplitudes,
                    self.seed, self.min_height
                ));
            }
        }
        self.chunks = new_chunks;
    }

    // create new chunks if they are visible but not yet present
    // if there are too many chunks currently in memory, run set_chunks_and_strip_old instead
    fn set_chunks(&mut self, camera: &Camera) {
        if self.chunks.len() > MAX_CHUNKMAP_SIZE {
            console_log!("Culling old chunks");
            self.set_chunks_and_strip_old(camera);
            return;
        }
        for (x, y) in camera.in_view(HEIGHTMAP_CHUNK_SIZE) {
            match self.chunks.get(&[x, y]) {
                None => {
                    self.chunks.insert([x, y], HeightmapChunk::new(
                        [x, y],
                        &self.periods, &self.amplitudes,
                        self.seed, self.min_height
                    ));
                },
                _ => (),
            }
        }
    }

    pub fn draw(&mut self, camera: &Camera) -> Canvas {
        let mut slices = SliceMap::new();
        self.set_chunks(&camera);
        let chunks = self.chunks.iter().map(|(_, c)| c).collect();
        slices.process_hchunks(camera, chunks);
        let mut canvas = Canvas::new(camera.height, camera.width);
        for (_, slice) in slices.0.into_iter() {
            slice.draw(&camera.proj_matrix, camera.origin, &mut canvas);
        }

        canvas
    }
}
