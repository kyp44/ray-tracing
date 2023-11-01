#![feature(cmp_minmax)]

use crate::hittable::{HittableList, Sphere};
use camera::Camera;
use clap::Parser;
use math::Point;
use num::rational::Ratio;

mod camera;
mod hittable;
mod image;
mod math;

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

    // Objects in the world
    let world = HittableList::new(Box::new([
        Box::new(Sphere::new(Point::new(0., 0., -1.), 0.5)),
        Box::new(Sphere::new(Point::new(0., -100.5, -1.), 100.)),
    ]));

    // Render image
    println!("{}", camera.render(&world));
}
