use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;

use tokio::time::{sleep, Duration};

use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width() as u32, capturer.height() as u32);

    let client = reqwest::Client::builder().http1_title_case_headers().build().unwrap();

    let config_json = fs::read_to_string("config.json").expect("Unable to read file");
    let config: serde_json::Value = serde_json::from_str(&config_json)?;
    let ip: &str = config["ip"].as_str().unwrap();
    let entity: &str = config["entity"].as_str().unwrap();
    let key: &str = config["key"].as_str().unwrap();

    println!("Loaded config. IP: {}, Entity: {}, Key: {}", ip, entity, key);

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

                let b = buffer[i] as u32;
                let g = buffer[i + 1] as u32;
                let r = buffer[i + 2] as u32;

                ar += r;
                ag += g;
                ab += b;
            }
        }

        ar /= w * h;
        ag /= w * h;
        ab /= w * h;

        // Send to HA
        let data = format!("{{\"entity_id\": \"{}\", \"rgb_color\": [{},{},{}]}}", entity, ar, ag, ab);
        let res = client.post(format!("http://{}/api/services/light/turn_on", ip))
            .body(data.clone())
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", key))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send().await?;

        // Sleep
        sleep(Duration::from_millis(20)).await;
    }
}
