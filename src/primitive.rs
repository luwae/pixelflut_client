#[derive(Debug, Copy, Clone)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
    pub color: (u8, u8, u8),
}

pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}
