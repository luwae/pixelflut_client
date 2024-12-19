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
        zn = mandel_iter(zn, c);
        if zn.r * zn.r + zn.c * zn.c > thresh {
            return Some(i);
        }
    }
    None
}

pub fn draw(rmin: f64, rmax: f64, imin: f64, imax: f64, xsize: usize, ysize: usize) -> Vec<Pixel> {
    let mut v = Vec::new();
    for x in 0..xsize {
        for y in 0..ysize {
            let coord = Complex::from(
                rmin + (rmax - rmin) * (x as f64 / xsize as f64),
                imin + (imax - imin) * ((ysize - (y + 1)) as f64 / ysize as f64),
            );
            if let Some(a) = mandel_exceeds(coord, 20, 100000.0) {
                v.push(Pixel {
                    x,
                    y,
                    color: (10 * a as u8, 10 * a as u8, 10 * a as u8),
                })
            } else {
                v.push(Pixel {
                    x,
                    y,
                    color: (255, 255, 255),
                })
            }
        }
    }
    v
}
