use cgmath::Vector3;
use derive_new::new;
use easy_cast::ConvFloat;
use num::{rational::Ratio, ToPrimitive};

const MAX_COLOR_CHANNEL: u8 = 255;

#[derive(Debug, Clone, Copy, new)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}
impl<T> Size<T>
where
    T: std::ops::Mul<T, Output = T> + num::Integer + ToPrimitive + num::bigint::ToBigInt + Copy,
{
    pub fn len(&self) -> T {
        self.width * self.height
    }

    pub fn aspect_ratio(&self) -> f64 {
        Ratio::new(self.width, self.height).to_f64().unwrap()
    }
}

type Channel = f64;
pub type Color = Vector3<Channel>;

struct ColorDisplay(Color);
impl ColorDisplay {
    fn apply_gamma_correction(x: f64) -> f64 {
        x.sqrt()
    }
}
impl std::fmt::Display for ColorDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.0.map(|x| {
            u8::conv_nearest(Channel::from(MAX_COLOR_CHANNEL) * Self::apply_gamma_correction(x))
        });

        write!(f, "{} {} {}", color.x, color.y, color.z,)
    }
}

#[derive(new)]
pub struct Image {
    size: Size<usize>,
    // Pixel colors in row major order
    pixel_data: Box<[Color]>,
}
impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print header: P3 format, image size, and max color value
        writeln!(
            f,
            "P3\n{} {}\n{MAX_COLOR_CHANNEL}",
            self.size.width, self.size.height
        )?;

        // Print pixel colors
        for color in self.pixel_data.iter() {
            writeln!(f, "{}", ColorDisplay(*color))?;
        }

        Ok(())
    }
}
