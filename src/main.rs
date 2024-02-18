use std::io::{Read, Write};
use std::net::TcpStream;
use fastrand;

mod primitive;
use primitive::{Pixel, Rect};

mod mandel;

mod tree;
use tree::{WormMutate, Worm, WormResult, DefaultMutate};

mod magnet;
use magnet::Particle;

#[derive(Debug)]
struct ServerInfo {
    width: u32,
    height: u32,
    recv_buffer_size: u32,
    send_buffer_size: u32,
}

fn decode_u32(data: &[u8]) -> u32 {
    (data[0] as u32)
        | ((data[1] as u32) << 8)
        | ((data[2] as u32) << 16)
        | ((data[3] as u32) << 24)
}

fn command_info(stream: &mut TcpStream) -> std::io::Result<ServerInfo> {
    let mut data = [0u8; 8];
    data[0] = b'I';
    stream.write_all(&data[..])?;
    let mut response = [0u8; 16];
    stream.read_exact(&mut response[..])?;
    let width = decode_u32(&response[0..4]);
    let height = decode_u32(&response[4..8]);
    let recv_buffer_size = decode_u32(&response[8..12]);
    let send_buffer_size = decode_u32(&response[12..16]);
    Ok(ServerInfo { width, height, recv_buffer_size, send_buffer_size })
}

fn command_print(px: &Pixel, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut data = [0u8; 8];
    data[0] = b'P';
    data[1] = px.x as u8;
    data[2] = (px.x >> 8) as u8;
    data[3] = px.y as u8;
    data[4] = (px.y >> 8) as u8;
    data[5] = px.color.0;
    data[6] = px.color.1;
    data[7] = px.color.2;
    stream.write_all(&data[..])?;
    Ok(())
}

fn encode_rect(rect: Rect, data: &mut [u8]) {
    // skip first byte
    data[1] = rect.x as u8;
    data[2] = (rect.x >> 8) as u8;
    data[3] = rect.y as u8;
    data[4] = (rect.y >> 8) as u8;
    data[5] = rect.w as u8;
    data[6] = rect.h as u8;
    data[7] = ((rect.w >> 8) & 0x0f) as u8 | ((rect.h >> 4) & 0xf0) as u8;
}

fn command_rectangle_get(colors: &mut [(u8, u8, u8)], rect: Rect, stream: &mut TcpStream) -> std::io::Result<()> {
    assert!(colors.len() == rect.w * rect.h);
    let mut command: [u8; 8] = [0; 8];
    command[0] = b'g';
    encode_rect(rect, &mut command[..]);
    stream.write_all(&command[..])?;
    // receive pixels
    let mut data: Box<[u8; 1024]> = Box::new([0; 1024]);
    let mut num_bytes_to_read: usize = rect.w * rect.h * 4;
    let mut pixel_idx = 0;
    while num_bytes_to_read > 0 {
        let mut read_size = num_bytes_to_read;
        if read_size > 1024 {
            read_size = 1024;
        }
        stream.read_exact(&mut data[0..read_size])?;
        num_bytes_to_read -= read_size;
        for i in (0..read_size).step_by(4) {
            colors[pixel_idx] = (data[i + 0], data[i + 1], data[i + 2]);
            pixel_idx += 1;
        }
    }
    Ok(())
}

fn command_rectangle_print(colors: &[(u8, u8, u8)], rect: Rect, stream: &mut TcpStream) -> std::io::Result<()> {
    assert!(colors.len() == rect.w * rect.h);
    let mut data: Box<[u8; 1024]> = Box::new([0; 1024]);
    // first round: write actual command
    data[0] = b'p';
    encode_rect(rect, &mut data[0..8]);
    let mut data_fill_start: usize = 8;
    let mut pixel_idx = 0;
    while pixel_idx < colors.len() {
        // fill buffer
        while data_fill_start <= 1024 - 4 && pixel_idx < colors.len() {
            let col = colors[pixel_idx];
            data[data_fill_start] = col.0;
            data[data_fill_start + 1] = col.1;
            data[data_fill_start + 2] = col.2;
            data[data_fill_start + 3] = 0;
            pixel_idx += 1;
            data_fill_start += 4;
        }
        stream.write_all(&data[0..data_fill_start])?; // buffer may not be full in last round
        data_fill_start = 0; // reset buffer
    }
    Ok(())
}

fn command_rectangle_fill(color: (u8, u8, u8), rect: Rect, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut data = [0u8; 12];
    // first round: write actual command
    data[0] = b'f';
    encode_rect(rect, &mut data[0..8]);
    data[8] = color.0;
    data[9] = color.1;
    data[10] = color.2;
    stream.write_all(&data[..])?;
    Ok(())
}

fn add_delta_single(delta: i32, top: i32, bot: i32, base: u8) -> u8 {
    let m: i32 = delta * top / bot + (base as i32);
    if m < 0 {
        0
    } else if m > 255 {
        255
    } else {
        m as u8
    }
}

fn add_delta(delta: (i32, i32, i32), top: i32, bot: i32, base: &mut (u8, u8, u8)) {
    base.0 = add_delta_single(delta.0, top, bot, base.0);
    base.1 = add_delta_single(delta.1, top, bot, base.1);
    base.2 = add_delta_single(delta.2, top, bot, base.2);
}

fn approx_single(col: u8) -> (u8, i32) {
    if col < 128 {
        (0, col as i32)
    } else {
        (255, col as i32 - 255)
    }
}

fn approx(col: (u8, u8, u8)) -> ((u8, u8, u8), (i32, i32, i32)) {
    let light: u8 = ((col.0 as i32 + col.1 as i32 + col.2 as i32) / 3) as u8;
    let (a, d) = approx_single(light);
    ((a, a, a), (d, d, d))
}

/*        *  7/16
 * 3/16 5/16 1/16
 */

struct Screen {
    w: usize,
    h: usize,
    colors: Vec<(u8, u8, u8)>,
}

impl Screen {
    fn get_neighbor_mut(&mut self, x: usize, y: usize, dx: i32, dy: i32) -> Option<&mut (u8, u8, u8)> {
        let new_x = if dx >= 0 && x + (dx as usize) < self.w {
            Some(x + (dx as usize))
        } else if dx < 0 && (x as i32 + dx) >= 0 {
            Some((x as i32 + dx) as usize)
        } else {
            None
        };
        let new_y = if dy >= 0 && y + (dy as usize) < self.h {
            Some(y + (dy as usize))
        } else if dy < 0 && (y as i32 + dy) >= 0 {
            Some((y as i32 + dy) as usize)
        } else {
            None
        };
        if let (Some(nx), Some(ny)) = (new_x, new_y) {
            Some(&mut self.colors[ny * self.w + nx])
        } else {
            None
        }
    }
}

fn floyd_steinberg_bw(rect: Rect, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut colors = vec![(0u8, 0u8, 0u8); (rect.w as usize) * (rect.h as usize)];
    command_rectangle_get(&mut colors[..], rect, stream)?;
    for y in rect.ys_abs() {
        for x in rect.xs_abs() {
            let (new_col, delta) = approx(colors[rect.index_abs(x, y)]);
            colors[rect.index_abs(x, y)] = new_col;
            // distribute delta
            if let Some((xx, yy)) = rect.get_mut_with_delta_abs(x, y, 1, 0) {
                add_delta(delta, 7, 16, &mut colors[rect.index_abs(xx, yy)]);
            }
            if let Some((xx, yy)) = rect.get_mut_with_delta_abs(x, y, -1, 1) {
                add_delta(delta, 3, 16, &mut colors[rect.index_abs(xx, yy)]);
            }
            if let Some((xx, yy)) = rect.get_mut_with_delta_abs(x, y, 0, 1) {
                add_delta(delta, 5, 16, &mut colors[rect.index_abs(xx, yy)]);
            }
            if let Some((xx, yy)) = rect.get_mut_with_delta_abs(x, y, 1, 1) {
                add_delta(delta, 1, 16, &mut colors[rect.index_abs(xx, yy)]);
            }
        }
    }
    
    command_rectangle_print(&colors[..], rect, stream)?;
    Ok(())
}

fn clamp(i: i32) -> u8 {
    if i < 0 {
        0
    } else if i > 255 {
        255
    } else {
        i as u8
    }
}

fn kernel_3x3(rect: Rect, kernel: [(i32, i32); 9], stream: &mut TcpStream) -> std::io::Result<()> {
    let mut colors = vec![(0u8, 0u8, 0u8); (rect.w as usize) * (rect.h as usize)];
    let mut new_colors = vec![(0u8, 0u8, 0u8); (rect.w as usize) * (rect.h as usize)];
    command_rectangle_get(&mut colors[..], rect, stream)?;

    for y in rect.ys_abs() {
        for x in rect.xs_abs() {
            let mut new_color = (0i32, 0i32, 0i32);
            let mut index = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (top, bottom) = kernel[index];
                    if let Some((xx, yy)) = rect.get_mut_with_delta_abs(x, y, dx, dy) {
                        let color = colors[rect.index_abs(xx, yy)];
                        new_color.0 += color.0 as i32 * top / bottom;
                        new_color.1 += color.1 as i32 * top / bottom;
                        new_color.2 += color.2 as i32 * top / bottom;
                    }
                    index += 1;
                }
            }
            new_colors[rect.index_abs(x, y)] = (clamp(new_color.0), clamp(new_color.1), clamp(new_color.2));
        }
    }

    command_rectangle_print(&new_colors[..], rect, stream)?;
    Ok(())
}

fn draw_circle_eighth(center: (usize, usize), radius: usize) -> Vec<(usize, usize)> {
    let rsq = radius * radius;
    let mut coords = Vec::new();
    let (mut x, mut y) = (center.0, center.1 + radius);
    let (mut from_c_x, mut from_c_y) = (0, radius);
    coords.push((x, y));
    while from_c_y >= from_c_x {
        let dx1 = (x + 1) - center.0;
        let dy1 = y - center.1;
        let d1 = dx1 * dx1 + dy1 * dy1;
        let discrepancy1 = if d1 < rsq { rsq - d1 } else { d1 - rsq};
        let dx2 = (x + 1) - center.0;
        let dy2 = (y - 1) - center.1;
        let d2 = dx2 * dx2 + dy2 * dy2;
        let discrepancy2 = if d2 < rsq { rsq - d2 } else { d2 - rsq};
        if discrepancy1 <= discrepancy2 {
            x += 1;
            from_c_x += 1;
        } else {
            x += 1;
            from_c_x += 1;
            y -= 1;
            from_c_y -= 1;
        }
        coords.push((x, y));
    }
    coords
}

fn draw_circle(center: (usize, usize), radius: usize) -> Vec<(usize, usize)> {
    let mut coords = draw_circle_eighth(center, radius);
    let mut coords2: Vec<(usize, usize)> = coords.iter().map(|(x, y)|
        (center.0 + y - center.1, center.1 + x - center.0)
        ).collect();
    coords.append(&mut coords2);

    let mut coords2: Vec<(usize, usize)> = coords.iter().map(|(x, y)|
        (*x, center.1 - (y - center.1))
        ).collect();
    coords.append(&mut coords2);

    let mut coords2: Vec<(usize, usize)> = coords.iter().map(|(x, y)|
        (center.0 - (x - center.0), *y)
        ).collect();
    coords.append(&mut coords2);

    coords
}

// TODO worm

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1337")?;

    let info = command_info(&mut stream)?;
 
    command_rectangle_fill((0, 0, 0), Rect { x: 0, y: 0, w: info.width as usize, h: info.height as usize }, &mut stream)?;

    /*
    let mut worms = Vec::new();
    let mut worms2 = Vec::new();
    worms.push(Worm::from((info.width as usize / 2) as f64, (info.height as usize - 1) as f64, std::f64::consts::PI / 8.0, 3.0, 20, (200, 200, 200)));
    while worms.len() > 0 {
        for mut worm in worms.drain(..) {
            if let Some(WormResult { mut new_worms, pixels }) = worm.step::<tree::Mutate2>(info.width as usize, info.height as usize) {
                for px in &pixels {
                    command_print(px, &mut stream)?;
                }
                worms2.push(worm);
                worms2.append(&mut new_worms);
            }
        }
        // the original worms is empty now
        std::mem::swap(&mut worms, &mut worms2);
    }

    let size = 15;
    let mut colors = vec![(0u8, 0u8, 0u8); size*size];
    loop {
        let rx = fastrand::usize(0..(info.width as usize));
        let ry = fastrand::usize(0..(info.height as usize));
        let rect = Rect { x: rx, y: ry, w: size, h: size };
        command_rectangle_get(&mut colors[..], rect, &mut stream)?;
        let mut rr: usize = 0;
        let mut gg: usize = 0;
        let mut bb: usize = 0;
        for color in &colors {
            rr += color.0 as usize;
            gg += color.1 as usize;
            bb += color.2 as usize;
        }
        rr /= size * size;
        gg /= size * size;
        bb /= size * size;
        command_rectangle_fill((rr as u8, gg as u8, bb as u8), rect, &mut stream)?;
    }
    */

    let create_obstacle = || (fastrand::f64() * info.width as f64, fastrand::f64() * info.height as f64);
    let nob = 10;
    let obstacles: Vec<(f64, f64)> = (0..nob).map(|_| create_obstacle()).collect();
    //let obstacles: Vec<(f64, f64)> = vec![(512.0, 100.0), (100.0, 512.0), (924.0, 512.0), (512.0, 924.0)];
    //for (ox, oy) in &obstacles {
    //    command_print(&Pixel { x: *ox as usize, y: *oy as usize, color: (255,0,0) }, &mut stream)?;
    //}
    for nn in 0..nob {
        let n = 50;
        for i in 0..n {
            let dx = (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos();
            let dy = -(2.0 * std::f64::consts::PI * i as f64 / n as f64).sin();
            let mut p = Particle::stationary(8.0*dx + obstacles[nn].0, 8.0*dy + obstacles[nn].1);
            let mut skip: Option<usize> = None;
            let delta_stop = fastrand::f64() * 20.0;
            let mut steps: usize = 0;
            while p.x >= 0.0 && p.y >= 0.0 && p.x <= info.width as f64 && p.y <= info.height as f64 {
                p.step(&obstacles[..]);
                let toc_x = p.x - info.width as f64 / 2.0;
                let toc_y = p.y - info.height as f64 / 2.0;
                skip = None;
                /*
                skip = match skip {
                    Some(i) if i > 0 => Some(i - 1),
                    Some(_) => None,
                    None => {
                        if fastrand::f64() < 0.1 {
                            Some(fastrand::usize(2..8))
                        } else {
                            None
                        }
                    },
                };
                */
                if (toc_x*toc_x + toc_y*toc_y).sqrt() <= info.width as f64 / 2.0 - delta_stop && skip.is_none() {
                    let c = if steps > 255*2 { 0u8 } else { (255 - steps / 2) as u8 };
                    command_print(&Pixel { x: p.x as usize, y: p.y as usize, color: (c,c,c) }, &mut stream)?;
                }
                steps += 1;
                // std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }

    Ok(())
}
