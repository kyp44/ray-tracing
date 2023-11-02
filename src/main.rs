#![feature(cmp_minmax)]

use crate::{
    hittable::{Hittable, HittableList, Sphere},
    image::Color,
    material::{Lambertian, Metal},
};
use camera::Camera;
use clap::Parser;
use math::Point;
use num::rational::Ratio;

mod camera;
mod hittable;
mod image;
mod material;
mod math;

/// This needs to be a particular type and not parametrized using the [`Rng`](rand::Rng) trait because we need trait objects.
type UsedRng = rand::rngs::ThreadRng;

/// A basic ray tracer, following the 'Ray Tracing in One Weekend' series of books.
/// Prints PPM image text.
#[derive(Parser, Debug)]
#[command(author, about)]
struct Args {
    /// Render image width, with the height being determined by a 16:9 aspect ratio.
    #[arg(short = 'w', long, default_value_t = 400)]
    image_width: usize,
}

fn main() {
    // Parse arguments
    let args = Args::parse();

    // Setup camera
    let camera = Camera::new(args.image_width, Ratio::new(16, 9));

    // World objects
    let world = [
        Sphere::new(
            Point::new(0., -100.5, -1.),
            100.,
            Box::new(Lambertian::new(Color::new(0.8, 0.8, 0.))),
        ),
        Sphere::new(
            Point::new(0., 0., -1.),
            0.5,
            Box::new(Lambertian::new(Color::new(0.7, 0.3, 0.3))),
        ),
        Sphere::new(
            Point::new(-1., 0., -1.),
            0.5,
            Box::new(Metal::new(Color::new(0.8, 0.8, 0.8))),
        ),
        Sphere::new(
            Point::new(1., 0., -1.),
            0.5,
            Box::new(Metal::new(Color::new(0.8, 0.6, 0.2))),
        ),
    ];

    // Render image
    println!(
        "{}",
        camera.render(&HittableList::new(
            &world
                .iter()
                .map(|h| h as &dyn Hittable)
                .collect::<Box<[_]>>(),
        ))
    );
}
