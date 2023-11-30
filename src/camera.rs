use crate::{
    hittable::Hittable,
    image::{Color, Image, Size},
    math::{BasisVectors, DirectionVectors, Point, Ray, Vector, VectorExt},
    UsedRng,
};
use cgmath::{ElementWise, InnerSpace, VectorSpace, Zero};
use easy_cast::Cast;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::iproduct;
use num::rational::Ratio;
use rand::{thread_rng, Rng};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::ops::RangeInclusive;

/// The location of the focal point of the camera.
const CAMERA_LOOK_FROM: Point = Point::new(13., 2., 3.);
/// Point the center of the camera is aimed towards.
const CAMERA_LOOK_AT: Point = Point::new(0., 0., 0.);
/// Camera-relative up direction
const CAMERA_UP_DIRECTION: Vector = Vector::new(0., 1., 0.);
/// Vertical camera field of view in degrees.
const CAMERA_VERTICAL_FOV: f64 = 20.;
/// Number of random samples averaged to render a single pixel.
const SAMPLES_PER_PIXEL: usize = 500;
/// Variation angle of rays through each pixel in degrees.
const DEFOCUS_ANGLE: f64 = 0.6;
/// Distance from the camera look from point to the plane of perfect focus
const FOCUS_DISTANCE: f64 = 10.;
/// The maximum number of ray bounces before just being black.
const MAX_DEPTH: usize = 50;

pub struct Camera {
    image_size: Size<usize>,
    pixel_upper_left: Point,
    pixel_delta_vectors: DirectionVectors,
    defocus_disk_basis: DirectionVectors,
}
impl Camera {
    pub fn new(image_width: usize, aspect_ratio: Ratio<usize>) -> Self {
        // Calculate the image size
        let image_size = Size::new(
            image_width,
            (Ratio::from(image_width) / aspect_ratio).to_integer(),
        );

        // Determine viewport height and size.
        let viewport_height = 2. * FOCUS_DISTANCE * (CAMERA_VERTICAL_FOV.to_radians() / 2.).tan();
        let viewport_size = Size::new(image_size.aspect_ratio() * viewport_height, viewport_height);

        // Determine the camera basis vectors
        let w = (CAMERA_LOOK_FROM - CAMERA_LOOK_AT).normalize();
        let u = CAMERA_UP_DIRECTION.cross(w).normalize();
        let camera_frame_basis = BasisVectors::new(u, w.cross(u), w);

        // Set the viewport edge vectors
        let viewport_edge_vectors = DirectionVectors::new(
            viewport_size.width * camera_frame_basis.u,
            -viewport_size.height * camera_frame_basis.v,
        );

        // Set the pixel-to-pixel vectors
        let pixel_delta_vectors = DirectionVectors::new(
            viewport_edge_vectors.u / image_size.width.cast(),
            viewport_edge_vectors.v / image_size.height.cast(),
        );

        // Calculate the location of the upper left of the viewport
        let viewport_upper_left = CAMERA_LOOK_FROM
            - FOCUS_DISTANCE * camera_frame_basis.w
            - viewport_edge_vectors.u / 2.
            - viewport_edge_vectors.v / 2.;

        // Calculate upper left pixel location
        let pixel_upper_left =
            viewport_upper_left + 0.5 * (pixel_delta_vectors.u + pixel_delta_vectors.v);

        // Calculate camera defocus disk radii
        let defocus_radius = FOCUS_DISTANCE * (DEFOCUS_ANGLE.to_radians() / 2.).tan();
        let defocus_disk_basis = DirectionVectors::new(
            defocus_radius * camera_frame_basis.u,
            defocus_radius * camera_frame_basis.v,
        );

        Self {
            image_size,
            pixel_upper_left,
            pixel_delta_vectors,
            defocus_disk_basis,
        }
    }

    fn ray_color<H: Hittable>(rng: &mut UsedRng, depth: usize, ray: &Ray, hittable: &H) -> Color {
        // If we have recursed too much just return black
        if depth == 0 {
            return Color::zero();
        }

        // Did we hit something?
        match hittable.hit(ray, &RangeInclusive::new(0.001, f64::INFINITY)) {
            Some(hr) => {
                // Scatter based on the material
                let scatter = hr.material.scatter(rng, ray, &hr);
                match scatter.ray {
                    Some(r) => Self::ray_color(rng, depth - 1, &r, hittable)
                        .mul_element_wise(scatter.attenuation),
                    None => Color::zero(),
                }
            }
            None => {
                // Creates a sky-like color gradient
                let unit = ray.direction.normalize();
                Color::new(1., 1., 1.).lerp(Color::new(0.5, 0.7, 1.), 0.5 * (unit.y + 1.))
            }
        }
    }

    fn get_ray(&self, rng: &mut UsedRng, pixel_center: Point) -> Ray {
        // Get a random point on the defocus disk
        let ray_origin = {
            if DEFOCUS_ANGLE > 0. {
                let point = Vector::random_within_unit_disk(rng);
                CAMERA_LOOK_FROM
                    + point.x * self.defocus_disk_basis.u
                    + point.y * self.defocus_disk_basis.v
            } else {
                CAMERA_LOOK_FROM
            }
        };

        let pixel_sample = pixel_center
            + (rng.gen::<f64>() - 0.5) * self.pixel_delta_vectors.u
            + (rng.gen::<f64>() - 0.5) * self.pixel_delta_vectors.v;

        Ray::new(ray_origin, pixel_sample - ray_origin)
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

                // Create or retrieve the RNG for this thread
                let mut rng = thread_rng();

                // Project the ray from the camera through the pixel
                let pixel_center = self.pixel_upper_left
                    + self.pixel_delta_vectors.u * x.cast()
                    + self.pixel_delta_vectors.v * y.cast();

                // Average random sample point colors for anti-aliasing
                (
                    i,
                    Color::average((0..SAMPLES_PER_PIXEL).map(|_| {
                        let ray = self.get_ray(&mut rng, pixel_center);
                        Self::ray_color(&mut rng, MAX_DEPTH, &ray, hittable)
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
}
