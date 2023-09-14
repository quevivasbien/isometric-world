use std::ops::BitXor;
use itertools::iproduct;

use crate::Matrix;

fn inverse_probit(x: f32) -> f32 {
    -(1. / x - 1.).ln() / 1.702
}

fn simple_hash(x: i32) -> i32 {
    let x = x.wrapping_shl(16).bitxor(x) * 0x45d9f3b;
    let x = x.wrapping_shl(16).bitxor(x) * 0x45d9f3b;
    let x = x.wrapping_shl(16).bitxor(x);
    x
}

// hashes should be approximately normally distributed
// note that this is very crude since both my hash and my inverse probit approximant are crudely constructed,
// but it should be fine for the use case here
pub fn randn_hash(v: i32) -> f32 {
    let i = simple_hash(v);
    let f = ((i as f32 / i32::MAX as f32) + 1.) / 2.;
    inverse_probit(f)
}

fn smoothstep(x: f32) -> f32 {
    6. * x.powi(5) - 15. * x.powi(4) + 10. * x.powi(3)
}

fn interpolate(x0: f32, x1: f32, w: f32) -> f32 {
    x0 + smoothstep(w) * (x1 - x0)
}

struct Grads {
    grads: Matrix<(f32, f32)>,
    x0: i32, y0: i32, step: usize,
}

impl Grads {
    fn new(
        // height and width are size of noise block that grads overlay
        height: usize, width: usize,
        // x0 and y0 are origin of noise block
        x0: i32, y0: i32,
        // step is distance between grad nodes, must divide height and width
        step: usize,
        // random seed
        seed: i32,
    ) -> Self {
        assert!(height % step == 0 && width % step == 0);
        // console_log!("Initializing grads with x0 = {}, y0 = {}", x0, y0);
        let nrows = height / step;
        let ncols = height / step;
        let data = (0..nrows * ncols).map(
            |i| {
                let row = i / ncols;
                let col = i % ncols;
                let x = x0 + (col * step) as i32;
                let y = y0 + (row * step) as i32;
                let hashkey = x.wrapping_add(seed)
                                    .wrapping_mul(y.wrapping_sub(seed))
                                    .wrapping_mul(step as i32);
                (randn_hash(hashkey), randn_hash(-hashkey))
            }
        ).collect();

        Self {
            grads: Matrix::new(data, nrows as usize, ncols as usize),
            x0, y0, step,
        }
    }
    
    fn dotgrad(&self, x: f32, y: f32, xi: usize, yi: usize) -> f32 {
        let dx = x - (xi as f32);
        let dy = y - (yi as f32);
        // console_log!("getting ({}, {})", yi, xi);
        let (gx, gy) = self.grads.get(yi.min(self.grads.rows - 1), xi.min(self.grads.cols - 1));
        
        dx * gx + dy * gy
    }

    fn perlin_at(&self, x: i32, y: i32) -> f32 {
        // console_log!("perlin_at({}, {})", x, y);
        // normalize to units of grad indices
        let x = (x - self.x0) as f32 / self.step as f32;
        let y = (y - self.y0) as f32 / self.step as f32;
        // console_log!("({}, {})", x, y);

        let x0 = x as usize;
        let x1 = x0 + 1;
        let y0 = y as usize;
        let y1 = y0 + 1;

        let sx = x - (x0 as f32);
        let sy = y - (y0 as f32);

        let n0 = self.dotgrad(x, y, x0, y0);
        let n1 = self.dotgrad(x, y, x1, y0);
        let ix0 = interpolate(n0, n1, sx);

        let n2 = self.dotgrad(x, y, x0, y1);
        let n3 = self.dotgrad(x, y, x1, y1);
        let ix1 = interpolate(n2, n3, sx);

        interpolate(ix0, ix1, sy)
    }
}


fn perlin(
    height: usize, width: usize,
    x0: i32, y0: i32,
    step: usize, seed: i32,
) -> Matrix<f32> {
    let grads = Grads::new(height, width, x0, y0, step, seed);
    let ys = y0..(y0 + width as i32);
    let xs = x0..(x0 + width as i32);

    let data = iproduct!(ys, xs).map(
        |(y, x)| grads.perlin_at(x, y)
    ).collect();

    Matrix::<f32>::new(data, height as usize, width as usize)
}

pub fn perlin_layers(
    height: usize, width: usize,
    periods: Vec<usize>, amplitudes: Vec<f32>,  
    seed: i32
) -> Matrix<f32> {
    assert_eq!(periods.len(), amplitudes.len());
    periods.into_iter().zip(amplitudes.into_iter()).map(
        |(period, amplitude)| {
            let mut h = perlin(height, width, 0, 0, period, seed);
            h.data.iter_mut().for_each(|x| *x *= amplitude);
            h
        }
    ).reduce(
        |acc, h| {
            let new_data = acc.data.iter().zip(h.data.iter()).map(
                |(x_acc, x_new)| x_acc + x_new
            ).collect();
            Matrix::<f32>::new(new_data, acc.rows, acc.cols)
        }
    ).unwrap()
}
