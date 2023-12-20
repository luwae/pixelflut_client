use std::io::Write;
use std::net::TcpStream;
use fastrand;

mod primitive;
use primitive::Pixel;

mod mandel;

fn pixel_write(px: &Pixel, stream: &mut TcpStream) -> std::io::Result<()> {
    let data = format!("PX {} {} {:02x}{:02x}{:02x}\n", px.x, px.y,
        px.color.0, px.color.1, px.color.2);
    stream.write(data.as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("193.196.38.83:8000")?;

    let pixels = mandel::mandel_draw();
    loop {
        for pixel in pixels.iter() {
            pixel_write(&pixel, &mut stream)?;
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Ok(())
}
