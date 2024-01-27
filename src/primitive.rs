#[derive(Debug, Copy, Clone)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
    pub color: (u8, u8, u8),
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    pub fn get_mut_with_delta_abs(&self, x: usize, y: usize, dx: i32, dy: i32) -> Option<(usize, usize)> {
        let new_x = if dx >= 0 && x + (dx as usize) < self.x + self.w {
            Some(x + (dx as usize))
        } else if dx < 0 && (x as i32 + dx) >= self.x as i32 {
            Some((x as i32 + dx) as usize)
        } else {
            None
        };
        let new_y = if dy >= 0 && y + (dy as usize) < self.y + self.h {
            Some(y + (dy as usize))
        } else if dy < 0 && (y as i32 + dy) >= self.y as i32 {
            Some((y as i32 + dy) as usize)
        } else {
            None
        };
        if let (Some(nx), Some(ny)) = (new_x, new_y) {
            Some((nx, ny))
        } else {
            None
        }
    }

    pub fn index_abs(&self, x: usize, y: usize) -> usize {
        (y - self.y) * self.w + (x - self.x)
    }

    pub fn xs_abs(&self) -> std::ops::Range<usize> {
        self.x .. (self.x + self.w)
    }
    
    pub fn ys_abs(&self) -> std::ops::Range<usize> {
        self.y .. (self.y + self.h)
    }
}
