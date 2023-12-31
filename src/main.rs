use std::io::Write;
use std::net::TcpStream;
use fastrand;

mod primitive;
use primitive::Pixel;

mod mandel;

mod barnsley;

mod firework;

fn pixel_write(px: &Pixel, stream: &mut TcpStream) -> std::io::Result<()> {
    let data = format!("PX {} {} {:02x}{:02x}{:02x}\n", px.x, px.y,
        px.color.0, px.color.1, px.color.2);
    stream.write(data.as_bytes())?;
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
    let mut stream = TcpStream::connect("192.168.2.115:1337")?;
    /*
    let mut p = Pixel { x: 512, y: 512, color: (0, 0, 0) };
    loop {
        random_walk(&mut p);
        pixel_write(&p, &mut stream)?;
    }
    */
    loop {
        let mut f = firework::Firework::new(fastrand::f64() * 1024.0, fastrand::f64() * 1024.0, (255, 255, 255), (fastrand::bool(), fastrand::bool(), fastrand::bool()));

        loop {
            let v = f.current_pixels();
            if v.len() == 0 {
                break;
            }
            for px in v {
                pixel_write(&px, &mut stream)?;
            }
            f.step();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    Ok(())
}
