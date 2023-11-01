use crate::{
    hittable::Hittable,
    image::{Color, Image, Size},
    math::{DirectionVectors, Point, Ray, Vector, VectorExt},
};
use cgmath::{InnerSpace, VectorSpace, Zero};
use easy_cast::Cast;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::iproduct;
use num::rational::Ratio;
use rand::{thread_rng, Rng};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::ops::RangeInclusive;

const VIEWPORT_HEIGHT: f64 = 2.0;
const FOCAL_LENGTH: f64 = 1.0;
const CAMERA_CENTER: Point = Point::new(0., 0., 0.);
const SAMPLES_PER_PIXEL: usize = 50;
/// The maximum number of ray bounces before just being black.
const MAX_DEPTH: usize = 10;

pub struct Camera {
    image_size: Size<usize>,
    pixel_upper_left: Point,
    pixel_delta_vectors: DirectionVectors,
}
impl Camera {
    pub fn new(image_width: usize, aspect_ratio: Ratio<usize>) -> Self {
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
            viewport_edge_vectors.u / image_size.width.cast(),
            viewport_edge_vectors.v / image_size.height.cast(),
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
        }
    }

    pub fn render<H: Hittable>(&self, hittable: &H) -> Image {
        let image_size = self.image_size;

        // Render the scene
        let bar = ProgressBar::new(image_size.len().cast());
        bar.set_message("Rendering image...");
        bar.set_style(
            ProgressStyle::with_template("{msg}\n{percent}% {bar:60} [ETA: {eta_precise}]")
                .unwrap(),
        );
        let mut pixel_data = iproduct!(0..image_size.height, 0..image_size.width)
            .enumerate()
            .par_bridge()
            .map(|(i, (y, x))| {
                bar.inc(1);

                // Create or retrieve the RNG
                let mut rng = thread_rng();

                // Project the ray from the camera through the pixel
                let pixel_center = self.pixel_upper_left
                    + self.pixel_delta_vectors.u * x.cast()
                    + self.pixel_delta_vectors.v * y.cast();

                // Average random sample point colors for anti-aliasing
                (
                    i,
                    Color::average((0..SAMPLES_PER_PIXEL).map(|_| {
                        let pixel_sample = pixel_center
                            + (rng.gen::<f64>() - 0.5) * self.pixel_delta_vectors.u
                            + (rng.gen::<f64>() - 0.5) * self.pixel_delta_vectors.v;

                        let ray = Ray::new(CAMERA_CENTER, pixel_sample - CAMERA_CENTER);
                        self.ray_color(&mut rng, MAX_DEPTH, &ray, hittable)
                    })),
                )
            })
            .collect::<Vec<_>>();

        // Annoyingly, Rayon does not preserve order even when collecting, so we need to sort
        pixel_data.sort_by_key(|t| t.0);
        bar.finish_and_clear();

        Image::new(
            self.image_size,
            pixel_data.into_iter().map(|t| t.1).collect(),
        )
    }

    fn ray_color<R: Rng, H: Hittable>(
        &self,
        rng: &mut R,
        depth: usize,
        ray: &Ray,
        hittable: &H,
    ) -> Color {
        // If we have recursed too much just return black
        if depth == 0 {
            return Color::zero();
        }

        // Did we hit something?
        match hittable.hit(ray, &RangeInclusive::new(0.001, f64::INFINITY)) {
            Some(hr) => {
                // For diffusion, send out a random reflected ray until we do not hit any surfaces
                let direction = Vector::random_on_hemisphere(rng, hr.normal);
                self.ray_color(rng, depth - 1, &Ray::new(hr.point, direction), hittable) * 0.5
            }
            None => {
                let unit = ray.direction.normalize();
                Color::new(1., 1., 1.).lerp(Color::new(0.5, 0.7, 1.), 0.5 * (unit.y + 1.))
            }
        }
    }
}
