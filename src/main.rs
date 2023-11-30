#![feature(cmp_minmax)]

use crate::{
    hittable::{Hittable, HittableList, Sphere},
    image::Color,
    material::{Dielectric, Lambertian, Metal},
    math::VectorExt,
};
use camera::Camera;
use cgmath::{ElementWise, InnerSpace};
use clap::Parser;
use itertools::iproduct;
use math::Point;
use num::rational::Ratio;
use rand::{thread_rng, Rng};

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

    let mut world = vec![
        // Large ground sphere
        Sphere::new(
            Point::new(0., -1000., 0.),
            1000.,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        ),
    ];

    // Add random little spheres
    let mut rng = thread_rng();
    let gap_point = Point::new(4., 0.2, 0.);
    for (a, b) in iproduct!(-11..11, -11..11) {
        let center = Point::new(
            f64::from(a) + 0.9 * rng.gen::<f64>(),
            0.2,
            f64::from(b) + 0.9 * rng.gen::<f64>(),
        );

        if (center - gap_point).magnitude() > 0.9 {
            world.push(Sphere::new(
                center,
                0.2,
                match rng.gen::<f64>() {
                    x if x < 0.8 => {
                        // The squaring here ensures darker colors
                        let color = Color::random_unit_cube(&mut rng)
                            .mul_element_wise(Color::random_unit_cube(&mut rng));
                        Box::new(Lambertian::new(color))
                    }
                    x if x < 0.95 => {
                        let color = Color::random(&mut rng, 0.5..1.);
                        Box::new(Metal::new(color, 0.5 * rng.gen::<f64>()))
                    }
                    _ => Box::new(Dielectric::new(1.5)),
                },
            ))
        }
    }

    // Add constant large spheres
    world.extend([
        // Glass
        Sphere::new(Point::new(0., 1., 0.), 1., Box::new(Dielectric::new(1.5))),
        // Solid
        Sphere::new(
            Point::new(-4., 1., 0.),
            1.,
            Box::new(Lambertian::new(Color::new(0.4, 0.2, 0.1))),
        ),
        // Metal
        Sphere::new(
            Point::new(4., 1., 0.),
            1.,
            Box::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.)),
        ),
    ]);

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
