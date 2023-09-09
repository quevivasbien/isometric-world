use rand::{thread_rng, Rng};
use rand_distr::{StandardNormal, Uniform};

fn randn() -> f32 {
    thread_rng().sample(StandardNormal)
}

fn random() -> f32 {
    thread_rng().sample(Uniform::new(0., 1.))
}

fn linspace(start: f32, end: f32, length: usize) -> Vec<f32> {
    let step = (end - start) / ((length - 1) as f32);
    (0..length).scan(0., |state, _| {
        *state = *state + step;
        Some(*state)
    }).collect()
}

fn smoothstep(x: f32) -> f32 {
    6. * x.powi(5) - 15. * x.powi(4) + 10. * x.powi(3)
}

fn interpolate(x0: f32, x1: f32, w: f32) -> f32 {
    x0 + smoothstep(w) * (x1 - x0)
}

struct Grads {
    xdata: Vec<f32>,
    ydata: Vec<f32>,
    rows: usize,
    cols: usize,
}

impl Grads {
    fn new(rows: usize, cols: usize) -> Self {
        let xdata = (0..rows * cols).map(
            |_| randn()
        ).collect();
        let ydata = (0..rows * cols).map(
            |_| randn()
        ).collect();
 
        Self { xdata, ydata, rows, cols }
    }

    fn get(&self, xi: usize, yi: usize) -> (f32, f32) {
        let i = yi * self.cols + xi;
 
        (self.xdata[i], self.ydata[i])
    }

    fn dotgrad(&self, x: f32, y: f32, xi: usize, yi: usize) -> f32 {
        let dx = x - (xi as f32);
        let dy = y - (yi as f32);
        let (gx, gy) = self.get(xi, yi);
        
        dx * gx + dy * gy
    }

    fn perlin_at(&self, x: f32, y: f32) -> f32 {
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

pub struct Heightmap {
    pub data: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
}

fn perlin(height: usize, width: usize, grad_period: usize) -> Heightmap {
    let gradcols = 2 + (width / grad_period).max(0);
    let gradrows = 2 + (height / grad_period).max(0);
    let grads = Grads::new(gradrows, gradcols);
    let ys = linspace(1. + random(), (gradrows - 1) as f32 - random(), height);
    let xs = linspace(1. + random(), (gradcols - 1) as f32 - random(), width);

    let data = (0..height * width).map(
        |idx| {
            let (i, j) = (idx / width, idx % width);
            grads.perlin_at(xs[j], ys[i])
        }
    ).collect();

    Heightmap { data, rows: height, cols: width }
}

pub fn perlin_layers(height: usize, width: usize, periods: Vec<usize>, amplitudes: Vec<f32>) -> Heightmap {
    assert_eq!(periods.len(), amplitudes.len());
    periods.into_iter().zip(amplitudes.into_iter()).map(
        |(period, amplitude)| {
            let mut h = perlin(height, width, period);
            h.data.iter_mut().for_each(|x| *x *= amplitude);
            h
        }
    ).reduce(
        |acc, h| {
            let new_data = acc.data.iter().zip(h.data.iter()).map(
                |(x_acc, x_new)| x_acc + x_new
            ).collect();
            Heightmap { data: new_data, rows: acc.rows, cols: acc.cols }
        }
    ).unwrap()
}
