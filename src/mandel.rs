use crate::Pixel;

#[derive(Copy, Clone, Debug)]
struct Complex {
    r: f64,
    c: f64,
}

impl Complex {
    fn from(r: f64, c: f64) -> Self {
        Self { r, c }
    }

    fn add(mut self, other: Self) -> Self {
        self.r += other.r;
        self.c += other.c;
        self
    }

    fn mul(self, other: Self) -> Self {
        Self {
            r: self.r * other.r - self.c * other.c,
            c: self.r * other.c + self.c * other.r,
        }
    }
}

fn mandel_iter(zn: Complex, c: Complex) -> Complex {
    zn.mul(zn).add(c)
}

fn mandel_exceeds(c: Complex, max_iter: usize, thresh: f64) -> Option<usize> {
    let mut zn = Complex::from(0.0, 0.0);
    for i in 0..max_iter {
        zn = mandel_iter(zn ,c);
        if zn.r * zn.r + zn.c * zn.c > thresh {
            return Some(i);
        }
    }
    None
}

pub fn mandel_draw() -> Vec<Pixel> {
    let mut v = Vec::new();
    for i in -200i32..=100i32 {
        for j in -100i32..=100i32 {
            let j = -j;
            let coord = Complex::from(i as f64 / 100.0, j as f64 / 100.0);
            let x: usize = (i + 200).try_into().unwrap();
            let y: usize = (j + 100).try_into().unwrap();
            if let Some(a) = mandel_exceeds(coord, 20, 100000.0) {
                v.push(Pixel { x, y, color: (10*a as u8, 10*a as u8, 10*a as u8) })
            } else {
                v.push(Pixel { x, y, color: (255, 255, 255) })
            }
        }
    }
    v
}
