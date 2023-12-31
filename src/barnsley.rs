use fastrand;

use crate::primitive::Pixel;

pub fn barnsley_vec(n: usize) -> Vec<Pixel> {
    let mut p = Pixel {
        x: 0,
        y: 0,
        color: (200, 200, 40),
    };

    let mut v = Vec::new();
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut xn: f64 = 0.0;
    let mut yn: f64 = 0.0;
    for _ in 0..n {
       let num = fastrand::f64();
       if num < 0.01 {
           xn = 0.0;
           yn = 0.16 * y;
       } else if num < 0.86 {
           xn = 0.85 * x + 0.04 * y;
           yn = -0.04 * x + 0.85 * y + 1.6;
       } else if num < 0.93 {
           xn = 0.2 * x - 0.26 * y;
           yn = 0.23 * x + 0.22 * y + 1.6;
       } else {
           xn = -0.15 * x + 0.28 * y;
           yn = 0.26 * x + 0.24 * y + 0.44;
       }
       x = xn;
       y = yn;
       p.x = ((x+5.0)*50.0).round() as usize;
       p.y = ((y*50.0).round()) as usize;
       v.push(p);
    }

    v
}
