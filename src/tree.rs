use crate::primitive::Pixel;

pub trait WormMutate {
    fn delta_angle(worm: &Worm) -> f64;
    fn delta_size(worm: &Worm) -> isize;
    fn leaves(worm: &Worm) -> Vec<Pixel>;
    fn children(worm: &Worm) -> Vec<Worm>;
}

pub struct Worm {
    x: f64,
    y: f64,
    old_x: f64,
    old_y: f64,
    angle: f64,
    starting_angle: f64,
    velo: f64,
    size: usize,
    color: (u8, u8, u8),
}

pub struct WormResult {
    pub new_worms: Vec<Worm>,
    pub pixels: Vec<Pixel>,
}

impl Worm {
    pub fn from(x: f64, y: f64, angle: f64, velo: f64, size: usize, color: (u8, u8, u8)) -> Self {
        Self {
            x,
            y,
            old_x: x,
            old_y: y,
            angle,
            starting_angle: angle,
            velo,
            size,
            color,
        }
    }

    pub fn step<M: WormMutate>(&mut self, screen_width: usize, screen_height: usize) -> Option<WormResult> {
        let mut pixels = Vec::new();

        self.old_x = self.x;
        self.old_y = self.y;
        self.x += self.angle.cos() * self.velo;
        self.y -= self.angle.sin() * self.velo;
        self.angle += M::delta_angle(self);

        if self.x < 0.0 || self.y < 0.0
            || self.x > screen_width as f64 || self.y > screen_width as f64
        {
            return None;
        }

        let old_size = self.size;
        self.size = (self.size as isize + M::delta_size(self)) as usize; // TODO check underflow

        pixels.append(&mut M::leaves(self));
        
        if self.size < 4 {
            return None;
        }

        // draw big white circle
        pixels.append(&mut dc_pixels((self.x as usize, self.y as usize), self.size, self.color));
        // draw little black circle
        pixels.append(&mut dc_pixels((self.x as usize, self.y as usize), self.size - 1, (0, 0, 0)));
        // draw middle little black circle
        pixels.append(&mut dc_pixels((self.old_x as usize, self.old_y as usize), old_size - 1, (0, 0, 0)));
        return Some(WormResult { new_worms: M::children(self), pixels });
    }
}

pub fn dc_pixels(center: (usize, usize), radius: usize, color: (u8, u8, u8)) -> Vec<Pixel> {
    dc(center, radius).iter().map(|(xx, yy)| Pixel { x: *xx, y: *yy, color }).collect()
}

pub fn dc(center: (usize, usize), radius: usize) -> Vec<(usize, usize)> {
    if radius == 0 { panic!("radius 0?"); }
    if radius == 1 {
        return vec![center];
    }
    if radius == 2 {
        let mut coords: Vec<(isize, isize)> = vec![(0, -1), (-1, 0), (0, 0), (1, 0), (0, 1)];
        for c in &mut coords {
            c.0 += center.0 as isize;
            c.1 += center.1 as isize;
        }
        let ucoords: Vec<(usize, usize)> = coords.into_iter()
            .filter(|(x, y)| *x >= 0 && *y >= 0)
            .map(|(x, y)| (x as usize, y as usize))
            .collect();
        return ucoords;
    }
    if radius == 3 {
        let mut coords: Vec<(isize, isize)> = vec![
            (-1, -2), (0, -2), (1, -2),
            (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1),
            (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0),
            (-2, 1), (-1, 1), (0, 1), (1, 1), (2, 1),
            (-1, 2), (0, 2), (1, 2),
        ];
        for c in &mut coords {
            c.0 += center.0 as isize;
            c.1 += center.1 as isize;
        }
        let ucoords: Vec<(usize, usize)> = coords.into_iter()
            .filter(|(x, y)| *x >= 0 && *y >= 0)
            .map(|(x, y)| (x as usize, y as usize))
            .collect();
        return ucoords;
    }
    if radius == 4 {
        let mut coords: Vec<(isize, isize)> = vec![
            (-2, -3), (-1, -3), (0, -3), (1, -3), (2, -3),
            (-3, -2), (-2, -2), (-1, -2), (0, -2), (1, -2), (2, -2), (3, -2),
            (-3, -1), (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1), (3, -1),
            (-3, 0), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (3, 0),
            (-3, 1), (-2, 1), (-1, 1), (0, 1), (1, 1), (2, 1), (3, 1),
            (-3, 2), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2), (3, 2),
            (-2, 3), (-1, 3), (0, 3), (1, 3), (2, 3),
        ];
        for c in &mut coords {
            c.0 += center.0 as isize;
            c.1 += center.1 as isize;
        }
        let ucoords: Vec<(usize, usize)> = coords.into_iter()
            .filter(|(x, y)| *x >= 0 && *y >= 0)
            .map(|(x, y)| (x as usize, y as usize))
            .collect();
        return ucoords;
    }
    let mut coords = Vec::new();
    let (icx, icy) = (center.0 as isize, center.1 as isize);
    let ir = radius as isize;
    let lby = if icy - ir < 0 { 0 } else { icy - ir };
    let uby = icy + ir;
    let lbx = if icx - ir < 0 { 0 } else { icx - ir };
    let ubx = icx + ir;
    for y in lby..=uby {
        for x in lbx..=ubx {
            if (y - icy) * (y - icy) + (x - icx) * (x - icx) < ir * ir {
                if x >= 0 && y >= 0 {
                    coords.push((x as usize, y as usize));
                }
            }
        }
    }
    coords
}

pub struct DefaultMutate;

impl WormMutate for DefaultMutate {
    fn delta_angle(_worm: &Worm) -> f64 {
        let max_deviation = 0.5; // should be less than 2 pi
        let d = fastrand::f64() * max_deviation;
        d - max_deviation / 2.0
    }

    fn delta_size(worm: &Worm) -> isize {
        let additional_fac = if worm.size < 6 { -0.0 } else { 0.0 };
        if fastrand::f64() < 0.18 + additional_fac {
            -1
        } else {
            0
        }
    }

    fn leaves(worm: &Worm) -> Vec<Pixel> {
        let mut pixels = Vec::new();
        if worm.size < 6 {
            for _ in 0..8 {
                let leaf_dist = fastrand::f64() * 40.0;
                let leaf_angle = fastrand::f64() * 2.0 * std::f64::consts::PI;
                let mut x = worm.x as isize + (leaf_angle.cos() * leaf_dist) as isize;
                if x < 0 { x = 0; }
                let mut y = worm.y as isize - (leaf_angle.sin() * leaf_dist) as isize;
                if y < 0 { y = 0; }
                let red: u8 = fastrand::u8(10..20);
                let green: u8 = fastrand::u8(50..=255);
                let blue: u8 = fastrand::u8(0..10);
                let color = (green, red, blue);
                pixels.append(&mut dc_pixels((x as usize, y as usize), fastrand::usize(1..5), color));
            }
        }
        pixels
    }

    fn children(worm: &Worm) -> Vec<Worm> {
        let mut new_worms = Vec::new();
        if worm.size >= 4 {
            // create new worms
            // size is between 20 and 4
            // let additional_fac = (20 - self.size) as f64 / 100.0; // between 0.26 and 0.1
            let additional_fac = if worm.size < 6 { 0.1 } else { 0.0 };
            if fastrand::f64() < 0.03 + additional_fac {
                // goes either to the left or to the right
                let new_worm = Worm::from(worm.old_x, worm.old_y, worm.angle + if fastrand::bool() { 0.3 } else { -0.3 }, worm.velo, worm.size, worm.color);
                new_worms.push(new_worm);
            }
        }
        new_worms
    }
}
