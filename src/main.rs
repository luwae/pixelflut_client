use std::io::Write;
use std::net::TcpStream;
use fastrand;

mod primitive;
use primitive::Pixel;

mod mandel;

mod barnsley;

fn pixel_write(px: &Pixel, stream: &mut TcpStream) -> std::io::Result<()> {
    let data = format!("PX {} {} {:02x}{:02x}{:02x}\n", px.x, px.y,
        px.color.0, px.color.1, px.color.2);
    stream.write(data.as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("192.168.2.119:1337")?;

    let pixels = barnsley::barnsley_vec(100000);
    for pixel in pixels.iter() {
        pixel_write(&pixel, &mut stream)?;
    }

    Ok(())
}
