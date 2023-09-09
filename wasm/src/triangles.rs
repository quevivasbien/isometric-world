use std::cmp::Ordering;

use crate::{Vertex, Color, Canvas};


pub struct Triangle {
    vertices: [Vertex; 3],
    fill: Color,
}

impl Triangle {
    pub fn new(mut vertices: [Vertex; 3], fill: Color) -> Self {
        // sort vertices in ascending y order
        vertices.sort_unstable_by(|u, v| u[1].partial_cmp(&v[1]).unwrap_or(Ordering::Equal));
        Self { vertices, fill }
    }

    // I may have problems here if vertex coordinates are negative...

    fn draw_flat_bottom(&self, canvas: &mut Canvas) {
        // draw under the assumption that vertices 1 and 2 are at equal y value
        let [v0, v1, v2] = self.vertices;
        let invslope0 = (v1[0] - v0[0]) / (v1[1] - v0[1]);
        let invslope1 = (v2[0] - v0[0]) / (v2[1] - v0[1]);
        let mut curx0 = v0[0];
        let mut curx1 = v0[0];
        for scanline_y in (v0[1] as usize)..=(v1[1] as usize) {
            // draw line between curx0 and curx1 at current scanline
            for x in (curx0 as usize)..=(curx1 as usize) {
                canvas.set_pixel(x, scanline_y, &self.fill)
            }
            // advance curx0 and curx1
            curx0 += invslope0;
            curx1 += invslope1;
        }
    }
    fn draw_flat_top(&self, canvas: &mut Canvas) {
        // draw under the assumption that vertices 1 and 2 are at equal y value
        let [v0, v1, v2] = self.vertices;
        let invslope0 = (v2[0] - v0[0]) / (v2[1] - v0[1]);
        let invslope1 = (v2[0] - v1[0]) / (v2[1] - v1[1]);
        let mut curx0 = v2[0];
        let mut curx1 = v2[0];
        for scanline_y in ((v0[1] as usize)..=(v2[1] as usize)).rev() {
            // draw line between curx0 and curx1 at current scanline
            for x in (curx0 as usize)..=(curx1 as usize) {
                canvas.set_pixel(x, scanline_y, &self.fill)
            }
            // advance curx0 and curx1
            curx0 -= invslope0;
            curx1 -= invslope1;
        }
    }
    pub fn draw(&self, canvas: &mut Canvas) {
        let [v0, v1, v2] = self.vertices;
        // handle trivial case of flat-bottom triangle
        if v1[1] == v2[1] {
            self.draw_flat_bottom(canvas)
        }
        // handle trivial case of flat-top triangle
        else if v0[1] == v1[1] {
            self.draw_flat_top(canvas)
        }
        // split triangle into top-flat and bottom-flat sections
        else {
            let v3 = [
                v0[0] + ((v2[0] - v0[0]) * (v1[1] - v0[1]) / (v2[1] - v0[1])),
                v1[1],
            ];
            Triangle { vertices: [v0, v1, v3], fill: self.fill.clone() }.draw_flat_bottom(canvas);
            Triangle { vertices: [v1, v3, v2], fill: self.fill.clone() }.draw_flat_top(canvas);
        }
    }
}
