use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width() as u32, capturer.height() as u32);

    loop {
        // Wait until frame
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    sleep(Duration::from_millis(20)).await;
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        // Find average color (packed BGRA)
        let stride = buffer.len() / (h as usize);

        let mut ar: u32 = 0;
        let mut ag: u32 = 0;
        let mut ab: u32 = 0;

        for y in 0..h {
            for x in 0..w {
                let i = stride * (y as usize) + (4 * x as usize);

                let r = buffer[i] as u32;
                let g = buffer[i + 1] as u32;
                let b = buffer[i + 2] as u32;

                ar += r;
                ag += g;
                ab += b;
            }
        }

        ar /= w * h;
        ag /= w * h;
        ab /= w * h;

        // Send to HA
        //println!("Avg: {:?}", (ar, ag, ab));

        // Sleep
        sleep(Duration::from_millis(20)).await;
    }
}
