use std::collections::HashMap;

use crate::{Block, ProjectionMatrix, Vertex, Canvas, Color, triangles::Triangle, Pos3};

struct Slice<'a> {
    pos: Pos3,
    index: u8,
    parent: &'a Block,
}

#[derive(Eq, Hash, PartialEq)]
struct SliceKey(i32, i32, bool);

impl<'a> Slice<'a> {
    fn create(index: u8, parent: &'a Block) -> (SliceKey, Slice<'a>) {
        assert!(index < 6);
        let pos = if index == 0 || index == 5 {
            parent.origin
        } else if index == 1 {
            [parent.origin[0], parent.origin[1] - 1, parent.origin[2] - 1]
        } else if index == 4 {
            [parent.origin[0] - 1, parent.origin[1], parent.origin[2] - 1]
        } else {
            [parent.origin[0], parent.origin[1], parent.origin[2] - 1]
        };
        (SliceKey(pos[1], pos[2], index % 2 == 0), Self { pos, index, parent })
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

    fn draw(&self, proj_matrix: &ProjectionMatrix, offset: &Vertex, canvas: &mut Canvas) {
        let v3d = if self.points_right() {
            [
                self.pos,
                [self.pos[0] + 1, self.pos[1], self.pos[2]],
                [self.pos[0] + 1, self.pos[1], self.pos[2]],
            ]
        } else {
            [
                self.pos,
                [self.pos[0] + 1, self.pos[1] + 1, self.pos[2]],
                [self.pos[0], self.pos[1] + 1, self.pos[2]],
            ]
        };
        let mut vertices = [[0f32; 2]; 3];
        for i in 0..3 {
            let v = v3d[i];
            let v2d = [v[0] - v[2], v[1] - v[2]];
            let proj = proj_matrix.proj(v2d);
            vertices[i] = [proj[0] + offset[0], proj[1] + offset[1]]
        }
        Triangle::new(vertices, self.color()).draw(canvas);
    }
}

pub struct Scene {
    pub blocks: Vec<Block>,
}

impl Scene {
    fn draw(&self, proj_matrix: &ProjectionMatrix, offset: &Vertex, canvas: &mut Canvas) {
        let mut slices = HashMap::<SliceKey, Slice>::new();
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
        for (_, slice) in slices.into_iter() {
            slice.draw(proj_matrix, offset, canvas);
        }
    }
}