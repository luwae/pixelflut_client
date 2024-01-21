use std::io::{Read, Write};
use std::net::TcpStream;
use fastrand;

mod primitive;
use primitive::{Pixel, Rect};

mod mandel;

mod barnsley;

mod firework;

fn pixel_write_multi(colors: &[(u8, u8, u8)], rect: Rect, stream: &mut TcpStream) -> std::io::Result<()> {
    assert!(rect.w > 0 && rect.h > 0);
    assert!(colors.len() == rect.w * rect.h);
    let mut data: Box<[u8; 1024]> = Box::new([0; 1024]);
    // first round: write actual command
    data[0] = b'p';
    data[1] = rect.x as u8;
    data[2] = (rect.x >> 8) as u8;
    data[3] = rect.y as u8;
    data[4] = (rect.y >> 8) as u8;
    data[5] = rect.w as u8;
    data[6] = rect.h as u8;
    data[7] = ((rect.w >> 8) & 0x0f) as u8 | ((rect.h >> 4) & 0xf0) as u8;
    let mut data_fill_start: usize = 8;
    let mut pixel_idx = 0;
    while pixel_idx < colors.len() {
        // fill buffer
        while data_fill_start < 1024 && pixel_idx < colors.len() {
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

fn pixel_write_multi_onecolor(color: (u8, u8, u8), rect: Rect, stream: &mut TcpStream) -> std::io::Result<()> {
    assert!(rect.w > 0 && rect.h > 0);
    let mut data: Box<[u8; 1024]> = Box::new([0; 1024]);
    // first round: write actual command
    data[0] = b'p';
    data[1] = rect.x as u8;
    data[2] = (rect.x >> 8) as u8;
    data[3] = rect.y as u8;
    data[4] = (rect.y >> 8) as u8;
    data[5] = rect.w as u8;
    data[6] = rect.h as u8;
    data[7] = ((rect.w >> 8) & 0x0f) as u8 | ((rect.h >> 4) & 0xf0) as u8;
    let mut data_fill_start: usize = 8;
    let mut pixel_idx = 0;
    while pixel_idx < rect.w * rect.h {
        // fill buffer
        while data_fill_start < 1024 && pixel_idx < rect.w * rect.h {
            data[data_fill_start] = color.0;
            data[data_fill_start + 1] = color.1;
            data[data_fill_start + 2] = color.2;
            data[data_fill_start + 3] = 0;
            pixel_idx += 1;
            data_fill_start += 4;
        }
        stream.write_all(&data[0..data_fill_start])?; // buffer may not be full in last round
        data_fill_start = 0; // reset buffer
    }
    Ok(())
}

fn pixel_write(px: &Pixel, stream: &mut TcpStream) -> std::io::Result<()> {
    // let data = format!("PX {} {} {:02x}{:02x}{:02x}\n", px.x, px.y,
        // px.color.0, px.color.1, px.color.2);
    let mut data = [0u8; 8];
    data[0] = b'P';
    data[1] = px.x as u8;
    data[2] = (px.x >> 8) as u8;
    data[3] = px.y as u8;
    data[4] = (px.y >> 8) as u8;
    data[5] = px.color.0;
    data[6] = px.color.1;
    data[7] = px.color.2;
    if stream.write(&data[..])? != 8 {
        panic!("write != 8");
    }
    Ok(())
}

fn pixel_read(px: &mut Pixel, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut data = [0u8; 8];
    data[0] = b'G';
    data[1] = px.x as u8;
    data[2] = (px.x >> 8) as u8;
    data[3] = px.y as u8;
    data[4] = (px.y >> 8) as u8;
    if stream.write(&data[..])? != 8 {
        panic!("write != 8");
    }
    if stream.read(&mut data[..])? != 4 {
        panic!("read != 4");
    }
    px.color.0 = data[0];
    px.color.1 = data[1];
    px.color.2 = data[2];
    Ok(())
}

/*
fn rec(stream: &mut TcpStream, sx: usize, sy: usize, idepth: usize) {
    if idepth == 10 {
        return;
    }
    let size = 1 << (10 - idepth);
    let col = ((255 * idepth) / 10) as u8;
    let mut p = Pixel { x: 0, y: 0, color: (col, 0, col) };
    for x in sx..(sx+size) {
        for y in sy..(sy+size) {
            p.x = x;
            p.y = y;
            pixel_write(&p, stream).unwrap();
        }
    }
    rec(stream, sx + (size/2), sy, idepth + 1);
    rec(stream, sx, sy + (size/2), idepth + 1);
}
*/

fn random_walk(p: &mut Pixel) {
    let dir = fastrand::usize(0..4);
    match dir {
        0 => if p.y == 0 { p.y = 1023; } else { p.y -= 1; },
        1 => if p.x == 1023 { p.x = 0; } else { p.x += 1; },
        2 => if p.y == 1023 { p.y = 0; } else { p.y += 1; },
        3 => if p.x == 0 { p.x = 1023; } else { p.x -= 1; },
        _ => panic!("dir"),
    }
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1337")?;
    // let pxs = mandel::draw(-1.5, 1.5, -1.5, 1.5, 1024, 1024);
    /*
    loop {
        let pxs = barnsley::barnsley_vec(1000);
        for px in &pxs {
            pixel_write(&px, &mut stream)?;
        }
    }
    */
    /*
    let mut px = Pixel { x: 0, y: 0, color: (0, 0, 0) };
    loop {
        for x in 0..256 {
            for y in 0..256 {
                px.x = x;
                px.y = y;
                pixel_read(&mut px, &mut stream)?;
                px.color.0 = 255u8 - px.color.0;
                px.color.1 = 255u8 - px.color.1;
                px.color.2 = 255u8 - px.color.2;
                pixel_write(&px, &mut stream)?;
            }
        }
    }
    */
    /*
    let rect = Rect { x: 0, y: 0, w: 512, h: 512 };
    let mut colors: Vec<(u8, u8, u8)> = Vec::new();
    for i in 0u8..100u8 {
        colors.push((i, i, i));
    }
    pixel_write_multi(&colors[..], rect, &mut stream)?;
    */
    let rect = Rect { x: 0, y: 0, w: 512, h: 512 };
    pixel_write_multi_onecolor((255, 0, 0), rect, &mut stream)?;
    /*
    let mut px = Pixel { x: 0, y: 0, color: (0, 0, 255) };
    for y in 0..512 {
        for x in 0..512 {
            px.x = x;
            px.y = y;
            pixel_write(&px, &mut stream)?;
        }
    }
    */

    Ok(())
}
