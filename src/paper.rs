use crate::primitive::Pixel;
use fastrand;

pub fn dot_at(x: usize, y: usize, color: (u8, u8, u8)) -> Vec<Pixel> {
    let mut pixels = Vec::new();
    let x = x as isize;
    let y = y as isize;
    for dy in -2isize..2isize {
        for dx in -2isize..2isize {
            if fastrand::f64() < 0.4 && x + dx >= 0 && y + dy >= 0 {
                pixels.push(Pixel { x: (x + dx) as usize, y: (y + dy) as usize, color });
            }
        }
    }
    for dy in -1isize..1isize {
        for dx in -1isize..1isize {
            if x + dx >= 0 && y + dy >= 0 {
                pixels.push(Pixel { x: (x + dx) as usize, y: (y + dy) as usize, color });
            }
        }
    }
    pixels
}
