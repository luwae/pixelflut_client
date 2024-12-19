use std::io::prelude::*;
use std::net::TcpStream;

#[derive(Default)]
struct Pixel {
    x: usize,
    y: usize,
    r: u8,
    g: u8,
    b: u8,
}

fn main() -> std::io::Result<()> { 
    let mut stream = TcpStream::connect("193.196.38.206:1234")?;

    let mut px: Pixel = Default::default();
    let px_str = format!("PX {} {} {:02x}{:02x}{:02x}\n", px.x, px.y, px.r, px.g, px.b);
    for xx in 0..256 {
        for yy in 0..256 {
            px.x = xx;
            px.y = yy;
            px.r = xx as u8;
            px.g = yy as u8;
            px.b = xx as u8;
            let px_str = format!("PX {} {} {:02x}{:02x}{:02x}\n", px.x, px.y, px.r, px.g, px.b);
            stream.write(px_str.as_bytes())?;
        }
    }
    Ok(())
}
