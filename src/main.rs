use std::time::Duration;

use derive_new::new;
use easy_cast::ConvFloat;
use indicatif::ProgressBar;
use itertools::iproduct;
use num::{rational::Ratio, ToPrimitive};

struct Size {
    width: u16,
    height: u16,
}
impl Size {
    const fn new(width: u16, height: u16) -> Self {
        Size { width, height }
    }

    fn len(&self) -> u32 {
        u32::from(self.width) * u32::from(self.height)
    }
}

type Channel = f64;

#[derive(new)]
struct Color {
    red: Channel,
    green: Channel,
    blue: Channel,
}
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn convert_channel(v: Channel) -> u8 {
            u8::conv_nearest(Channel::from(MAX_COLOR) * v)
        }

        write!(
            f,
            "{} {} {}",
            convert_channel(self.red),
            convert_channel(self.green),
            convert_channel(self.blue)
        )
    }
}

const IMAGE_SIZE: Size = Size::new(256, 256);
const MAX_COLOR: u8 = 255;

fn main() {
    // Print header: P3 format, image size, and max color value
    println!(
        "P3\n{} {}\n{MAX_COLOR}",
        IMAGE_SIZE.width, IMAGE_SIZE.height
    );

    fn create_channel(n: u16, d: u16) -> Channel {
        Ratio::new(n, d).to_f64().unwrap()
    }

    let bar = ProgressBar::new(IMAGE_SIZE.len().into());
    for (y, x) in iproduct!(0..IMAGE_SIZE.height, 0..IMAGE_SIZE.width) {
        bar.inc(1);

        // Artificial delay to demonstrate the progress bar
        std::thread::sleep(Duration::from_micros(5));

        let color = Color::new(
            create_channel(x, u16::from(IMAGE_SIZE.width - 1)),
            create_channel(y, u16::from(IMAGE_SIZE.height - 1)),
            0.,
        );

        println!("{color}");
    }
    bar.finish_and_clear();
}
