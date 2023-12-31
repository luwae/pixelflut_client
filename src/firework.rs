use crate::primitive::Pixel;

pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub color: (u8, u8, u8),
    pub fast_falloff: (bool, bool, bool),
}

impl Particle {
    pub fn step(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        if self.fast_falloff.0 {
            self.color.0 = ((self.color.0 as usize)*4/5) as u8;
        } else {
            self.color.0 = ((self.color.0 as usize)*19/20) as u8;
        }
        if self.fast_falloff.1 {
            self.color.1 = ((self.color.1 as usize)*4/5) as u8;
        } else {
            self.color.1 = ((self.color.1 as usize)*19/20) as u8;
        }
        if self.fast_falloff.2 {
            self.color.2 = ((self.color.2 as usize)*4/5) as u8;
        } else {
            self.color.2 = ((self.color.2 as usize)*19/20) as u8;
        }
        self.vy += 0.1;
    }

    pub fn to_pixel(&self, xsize: usize, ysize: usize) -> Option<Pixel> {
        if self.x < 0.0 || self.y < 0.0 {
            None
        } else {
            let (x, y) = (self.x as usize, self.y as usize);
            if x >= 1024 || y >= 1024 {
                None
            } else {
                if self.color == (0, 0, 0) {
                    None
                } else {
                    Some(Pixel {
                        x,
                        y,
                        color: self.color,
                    })
                }
            }
        }
    }
}

pub struct Firework {
    pub x: f64,
    pub y: f64,
    pub color: (u8, u8, u8),
    pub fast_falloff: (bool, bool, bool),
    particles: Vec<Particle>,
}

impl Firework {
    pub fn new(x: f64, y: f64, color: (u8, u8, u8), fast_falloff: (bool, bool, bool)) -> Self {
        let mut particles = Vec::new();
        for _ in 0..100 {
            particles.push(Particle {
                x,
                y,
                vx: (fastrand::f64() - 0.5) * 5.0,
                vy: (fastrand::f64()) * -5.0,
                color,
                fast_falloff
            });
        }
        Self {
            x, y, color, fast_falloff,
            particles,
        }
    }

    pub fn step(&mut self) {
        for p in &mut self.particles {
            p.step();
        }
    }

    pub fn current_pixels(&self) -> Vec<Pixel> {
        let mut v = Vec::new();
        for p in &self.particles {
            if let Some(px) = p.to_pixel(1024, 1024) {
                v.push(px);
            }
        }
        v
    }
}
