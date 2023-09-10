use std::{collections::HashMap, hash::Hash};

use crate::{Vertex, Canvas, Color, triangles::Triangle, Pos2, Pos3, terrain::Heightmap, utils::{round_down, round_up}};

const THETA: f32 = std::f32::consts::FRAC_PI_6;
const CHUNK_SIZE: i32 = 16;

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

// a set of blocks within some rectangle in the x,y plane
struct Chunk {
    bounds: Bounds,
    blocks: Vec<Block>,
}

impl Chunk {
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
    // the point[origin[0], origin[1], 0] should be rendered at the top right corner of the display
    pub origin: Vertex,
    // screen dimensions in terms of pixels
    pub height: usize,
    pub width: usize,
    // pixels per block edge
    pub scale: f32,
    pub proj_matrix: ProjectionMatrix,
}

impl Camera {
    pub fn new(origin: Vertex, height: usize, width: usize, scale: f32) -> Self {
        Camera {
            origin,
            height, width,
            scale,
            proj_matrix: ProjectionMatrix::new(scale),
        }
    }

    // determines which chunks in an array of chunks are (at least partially) in view
    // assumes that hash map keys are same as chunk origins, and chunk bounds are divisible by CHUNK_SIZE
    fn in_view<'a>(&self, chunks: &'a HashMap<Pos2, Chunk>) -> Vec<&'a Chunk> {
        // first get coordinates of screen bounds at z = 0
        let [x0, y0] = self.origin;
        let inv_proj = self.proj_matrix.inverse();
        let top_left = inv_proj.proj(self.origin);
        let top_right = inv_proj.proj([x0 + self.width as f32, y0]);
        let bottom_left = inv_proj.proj([x0, y0 + self.height as f32]);
        let bottom_right = inv_proj.proj([x0 + self.width as f32, y0 + self.height as f32]);
        // take extreme values and round to surrounding multiples of CHUNK_SIZE
        let x_min = round_down(top_left[0] as i32, CHUNK_SIZE);
        let x_max = round_up(bottom_right[0] as i32, CHUNK_SIZE);
        let y_min = round_down(top_right[1] as i32, CHUNK_SIZE);
        let y_max = round_up(bottom_left[1] as i32, CHUNK_SIZE);
        let mut chunks_out = Vec::<&Chunk>::new();
        for x in (x_min..x_max).step_by(CHUNK_SIZE as usize) {
            for y in (y_min..y_max).step_by(CHUNK_SIZE as usize) {
                match chunks.get(&[x, y]) {
                    Some(chunk) => chunks_out.push(chunk),
                    None => (),
                }
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

    fn draw(&self, proj_matrix: &ProjectionMatrix, origin: &Vertex, canvas: &mut Canvas) {
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
        for i in 0..3 {
            let proj = proj_matrix.proj([v[i][0] as f32, v[i][1] as f32]);
            vertices[i] = [proj[0] - origin[0], proj[1] - origin[1]];
        }
        Triangle::new(vertices, self.color()).draw(canvas);
    }
}

pub struct Scene {
    chunks: HashMap<Pos2, Chunk>,
}

impl Scene {
    fn add(&mut self, b: Block) {
        let chunk_x0 = round_down(b.origin[0], CHUNK_SIZE);
        let chunk_y0 = round_down(b.origin[1], CHUNK_SIZE);
        if let Some(chunk) = self.chunks.get_mut(&[chunk_x0, chunk_y0]) {
            chunk.add(b).unwrap();
        } else {
            let mut chunk = Chunk::new(Bounds{ x: (chunk_x0, chunk_x0 + CHUNK_SIZE), y: (chunk_y0, chunk_y0 + CHUNK_SIZE) });
            chunk.add(b).unwrap();
            self.chunks.insert([chunk_x0, chunk_y0], chunk);
        }
    }

    pub fn from_heightmap(h: Heightmap, min_height: i32) -> Self {
        let mut scene = Scene {
            chunks: HashMap::new(),
        };
        h.data.iter().enumerate().for_each(
            |(idx, height)| {
                let (i, j) = (idx / h.cols, idx % h.cols);
                for z in min_height..=(*height as i32) {
                    let x = (256. * (1. / ((1 + z - min_height) as f32).powf(0.35))) as u8;
                    scene.add(Block {
                        origin: [j as i32, i as i32, z],
                        color: Color { r: x, g: x, b: x },
                    });
                }
            }
        );

        scene
    }

    pub fn draw(&self, camera: &Camera) -> Canvas {
        let mut slices = HashMap::<SliceKey, Slice>::new();
        for chunk in camera.in_view(&self.chunks) {
            chunk.process_slices(&mut slices)
        }
        let mut canvas = Canvas::new(camera.height, camera.width);
        for (_, slice) in slices.into_iter() {
            slice.draw(&camera.proj_matrix, &camera.origin, &mut canvas);
        }

        canvas
    }
}
