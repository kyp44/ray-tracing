use crate::{
    hittable::Hittable,
    math::{DirectionVectors, Point, Ray, Vector},
};
use cgmath::{InnerSpace, Vector3, VectorSpace};
use derive_new::new;
use easy_cast::ConvFloat;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::iproduct;
use num::{rational::Ratio, ToPrimitive};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::ops::RangeInclusive;

const VIEWPORT_HEIGHT: f64 = 2.0;
const FOCAL_LENGTH: f64 = 1.0;
const CAMERA_CENTER: Point = Point::new(0., 0., 0.);
const MAX_COLOR_CHANNEL: u8 = 255;
const SAMPLES_PER_PIXEL: usize = 50;

#[derive(Debug, Clone, Copy, new)]
struct Size<T> {
    width: T,
    height: T,
}
impl<T> Size<T>
where
    T: std::ops::Mul<T, Output = T> + num::Integer + ToPrimitive + num::bigint::ToBigInt + Copy,
{
    fn len(&self) -> T {
        self.width * self.height
    }

    fn aspect_ratio(&self) -> f64 {
        Ratio::new(self.width, self.height).to_f64().unwrap()
    }
}

type Channel = f64;
type Color = Vector3<Channel>;

struct ColorDisplay(Color);
impl ColorDisplay {
    fn average(colors: impl Iterator<Item = Color>) -> Self {
        let mut n: u32 = 1;
        let color_sum = colors
            .reduce(|a, b| {
                n += 1;
                a + b
            })
            .expect("The color iterator cannot be empty");

        // NOTE: The code in the book clamps the average channels, but this is actually unnecessary as
        // intervals are preserved when averaging.

        Self(color_sum / f64::from(n))
    }
}
impl std::fmt::Display for ColorDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn convert_channel(v: Channel) -> u8 {
            u8::conv_nearest(Channel::from(MAX_COLOR_CHANNEL) * v)
        }

        write!(
            f,
            "{} {} {}",
            convert_channel(self.0.x),
            convert_channel(self.0.y),
            convert_channel(self.0.z),
        )
    }
}

pub struct Camera {
    image_size: Size<u32>,
    pixel_upper_left: Point,
    pixel_delta_vectors: DirectionVectors,
    rng: ThreadRng,
}
impl Camera {
    pub fn new(image_width: u32, aspect_ratio: Ratio<u32>) -> Self {
        // Calculate the image size
        let image_size = Size::new(
            image_width,
            (Ratio::from(image_width) / aspect_ratio).to_integer(),
        );

        // Calculate the viewport size based on the image size
        let viewport_size = Size::new(image_size.aspect_ratio() * VIEWPORT_HEIGHT, VIEWPORT_HEIGHT);

        // Set the viewport edge vectors
        let viewport_edge_vectors = DirectionVectors::new(
            Vector::new(viewport_size.width, 0., 0.),
            Vector::new(0., -viewport_size.height, 0.),
        );

        // Set the pixel-to-pixel vectors
        let pixel_delta_vectors = DirectionVectors::new(
            viewport_edge_vectors.u / image_size.width.into(),
            viewport_edge_vectors.v / image_size.height.into(),
        );

        // Calculate the location of the upper left of the viewport
        let viewport_upper_left = CAMERA_CENTER
            - Vector::new(0., 0., FOCAL_LENGTH)
            - viewport_edge_vectors.u / 2.
            - viewport_edge_vectors.v / 2.;

        // Calculate upper left pixel location
        let pixel_upper_left =
            viewport_upper_left + 0.5 * (pixel_delta_vectors.u + pixel_delta_vectors.v);

        Self {
            image_size,
            pixel_upper_left,
            pixel_delta_vectors,
            rng: thread_rng(),
        }
    }

    pub fn render<H: Hittable>(&mut self, hittable: &H) {
        let image_size = self.image_size;

        // Print header: P3 format, image size, and max color value
        println!(
            "P3\n{} {}\n{MAX_COLOR_CHANNEL}",
            image_size.width, image_size.height
        );

        // Render the scene
        let bar = ProgressBar::new(image_size.len().into());
        bar.set_message("Rendering image...");
        bar.set_style(
            ProgressStyle::with_template("{msg}\n{percent}% {bar:60} [ETA: {eta_precise}]")
                .unwrap(),
        );
        for (y, x) in iproduct!(0..image_size.height, 0..image_size.width) {
            bar.inc(1);

            // Project the ray from the camera through the pixel
            let pixel_center = self.pixel_upper_left
                + f64::from(x) * self.pixel_delta_vectors.u
                + f64::from(y) * self.pixel_delta_vectors.v;

            // Average random sample point colors for anti-aliasing
            let average_color = ColorDisplay::average((0..SAMPLES_PER_PIXEL).map(|_| {
                let pixel_sample = pixel_center
                    + (self.rng.gen::<f64>() - 0.5) * self.pixel_delta_vectors.u
                    + (self.rng.gen::<f64>() - 0.5) * self.pixel_delta_vectors.v;

                let ray = Ray::new(CAMERA_CENTER, pixel_sample - CAMERA_CENTER);
                self.ray_color(&ray, hittable)
            }));

            println!("{}", average_color);
        }
        bar.finish_and_clear();
    }

    fn ray_color<H: Hittable>(&self, ray: &Ray, hittable: &H) -> Color {
        match hittable.hit(ray, &RangeInclusive::new(0., f64::INFINITY)) {
            Some(hr) => (Color::from(hr.normal) + Color::new(1., 1., 1.)) * 0.5,
            None => {
                let unit = ray.direction.normalize();
                Color::new(1., 1., 1.).lerp(Color::new(0.5, 0.7, 1.), 0.5 * (unit.y + 1.))
            }
        }
    }
}
