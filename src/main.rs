use std::io::Write;
use std::net::TcpStream;
use fastrand;

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
    // let mut stream = TcpStream::connect("127.0.0.1:1337")?;
    let mut p = Pixel {
        x: 0,
        y: 0,
        color: (0, 100, 0),
    };

    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut xn: f64 = 0.0;
    let mut yn: f64 = 0.0;
    for _ in 0..10000 {
       let num = fastrand::f64();
       if num < 0.01 {
           xn = 0.0;
           yn = 0.16 * y;
       } else if num < 0.86 {
           xn = 0.85 * x + 0.04 * y;
           yn = -0.04 * x + 0.85 * y + 1.6;
       } else if num < 0.93 {
           xn = 0.2 * x - 0.26 * y;
           yn = 0.23 * x + 0.22 * y + 1.6;
       } else {
           xn = -0.15 * x + 0.28 * y;
           yn = 0.26 * x + 0.24 * y + 0.44;
       }
       x = xn;
       y = yn;
       p.x = ((x+5.0)*50.0).round() as usize;
       p.y = ((y*50.0).round()) as usize;
       p.write(&mut stream)?;
    }

    Ok(())
}
