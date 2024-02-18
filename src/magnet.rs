pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

impl Particle {
    pub fn stationary(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
        }
    }

    pub fn from(x: f64, y: f64, vx: f64, vy: f64) -> Self {
        Self { x, y, vx, vy }
    }

    pub fn step(&mut self, obstacles: &[(f64, f64)]) {
        self.vx = 0.0; // TODO
        self.vy = 0.0;
        for (ox, oy) in obstacles {
            let dx = self.x - ox;
            let dy = self.y - oy;
            let dist = (dx*dx + dy*dy).sqrt();
            let strength = 1.0/(dist*dist);
            self.vx += strength * dx;
            self.vy += strength * dy;
        }
        let speed = (self.vx*self.vx + self.vy*self.vy).sqrt();
        self.vx /= speed;
        self.vy /= speed;
        self.x += self.vx;
        self.y += self.vy;

        // self.vx *= 0.9;
        // self.vy *= 0.9;
    }
}
