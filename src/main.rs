#![feature(cmp_minmax)]

use cgmath::{InnerSpace, Vector3, VectorSpace};
use derive_new::new;
use easy_cast::ConvFloat;
use indicatif::ProgressBar;
use itertools::iproduct;
use math::Point;
use num::{rational::Ratio, ToPrimitive};

use crate::{
    hittable::Sphere,
    math::{DirectionVectors, Ray, Vector},
};

mod hittable;
mod math;

const IMAGE_WIDTH: u32 = 400;
const VIEWPORT_HEIGHT: f64 = 2.0;
const FOCAL_LENGTH: f64 = 1.0;
const CAMERA_CENTER: Point = Point::new(0., 0., 0.);
const MAX_COLOR: u8 = 255;

#[derive(Debug, new)]
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
impl std::fmt::Display for ColorDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn convert_channel(v: Channel) -> u8 {
            u8::conv_nearest(Channel::from(MAX_COLOR) * v)
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

fn main() {
    // Calculate the image size
    let aspect_ratio = Ratio::new(16u32, 9);
    let image_size = Size::new(
        IMAGE_WIDTH,
        (Ratio::from(IMAGE_WIDTH) / aspect_ratio).to_integer(),
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

    // Print header: P3 format, image size, and max color value
    println!(
        "P3\n{} {}\n{MAX_COLOR}",
        image_size.width, image_size.height
    );

    fn ray_color(ray: &Ray) -> Color {
        match Sphere::new(Point::new(0., 0., -1.), 0.5).hit_color(ray) {
            Some(c) => c,
            None => {
                let unit = ray.direction.normalize();
                Color::new(1., 1., 1.).lerp(Color::new(0.5, 0.7, 1.), 0.5 * (unit.y + 1.))
            }
        }
    }

    // Render the scene
    let bar = ProgressBar::new(image_size.len().into());
    for (y, x) in iproduct!(0..image_size.height, 0..image_size.width) {
        bar.inc(1);

        // Project the ray from the camera through the pixel
        let pixel_center = pixel_upper_left
            + f64::from(x) * pixel_delta_vectors.u
            + f64::from(y) * pixel_delta_vectors.v;
        let ray = Ray::new(CAMERA_CENTER, pixel_center - CAMERA_CENTER);

        println!("{}", ColorDisplay(ray_color(&ray)));
    }
    bar.finish_and_clear();
}
