use std::io::{Read, Write};
use std::net::TcpStream;

mod primitive;
use primitive::{Pixel, Rect};

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

fn add_delta_single(delta: i32, top: i32, bot: i32, base: u8) -> u8 {
    let m: i32 = delta * top / bot + (base as i32);
    if (m < 0) {
        0
    } else if (m > 255) {
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
    if (col < 128) {
        (0, col as i32)
    } else {
        (255, col as i32 - 255)
    }
}

fn approx(col: (u8, u8, u8)) -> ((u8, u8, u8), (i32, i32, i32)) {
    let (ar, dr) = approx_single(col.0);
    let (ag, dg) = approx_single(col.1);
    let (ab, db) = approx_single(col.2);
    ((ar, ag, ab), (dr, dg, db))
}

/*        *  7/16
 * 3/16 5/16 1/16
 */

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1337")?;
    let info = command_info(&mut stream)?;
    let mut colors = vec![(0u8, 0u8, 0u8); (info.width as usize) * (info.height as usize)];
    command_rectangle_get(&mut colors[..], Rect { x: 0, y: 0, w: info.width as usize, h: info.height as usize }, &mut stream)?;
    
    let w = info.width as usize;
    let h = info.height as usize;
    
    for y in 0usize..h {
        for x in 0usize..w {
            let (new_col, delta) = approx(colors[y * w + x]);
            colors[y * w + x] = new_col;
            // distribute delta
            if x + 1 < w {
                add_delta(delta, 7, 16, &mut colors[y * w + (x + 1)]);
            }
            if x != 0 && y + 1 < h {
                add_delta(delta, 3, 16, &mut colors[(y + 1) * w + (x - 1)]);
            }
            if y + 1 < h {
                add_delta(delta, 5, 16, &mut colors[(y + 1) * w + x]);
            }
            if x + 1 < w && y + 1 < h {
                add_delta(delta, 1, 16, &mut colors[(y + 1) * w + (x + 1)]);
            }
        }
    }
    
    command_rectangle_print(&colors[..], Rect { x: 0, y: 0, w: info.width as usize, h: info.height as usize }, &mut stream)?;
    
    println!("{:?}", info);
    Ok(())
}
