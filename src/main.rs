use std::io::Write;
use std::net::TcpStream;

struct Pixel {
    x: usize,
    y: usize,
    color: (u8, u8, u8),
}

impl Pixel {
    fn write(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let data = format!("PX {} {} {:02x}{:02x}{:02x}\n", self.x, self.y,
            self.color.0, self.color.1, self.color.2);
        print!("{}", data);
        stream.write(data.as_bytes())?;
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("193.196.38.83:8000")?;
    let mut p = Pixel {
        x: 5,
        y: 5,
        color: (0, 0, 0),
    };
    let (cx, cy): (i32, i32) = (200, 200);
    for x in -10i32..=10i32 {
        for y in -10i32..=10i32 {
            let (xx, yy) = (cx + x, cy + y);
            let dist = x*x + y*y;
            if dist <= 100 {
                p.x = xx.try_into().unwrap();
                p.y = yy.try_into().unwrap();
                let c = ((dist * 255) / 100) as u8;
                p.color = (c, c, c);
                p.write(&mut stream)?;
            }
        }
    }
    Ok(())
}
