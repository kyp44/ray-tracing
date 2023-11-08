use crate::{
    hittable::HitRecord,
    image::Color,
    math::{Ray, Vector, VectorExt},
    UsedRng,
};
use cgmath::InnerSpace;
use derive_new::new;
use num::clamp;

pub struct Scatter {
    pub attenuation: Color,
    pub ray: Option<Ray>,
}

pub trait Material {
    fn scatter(&self, rng: &mut UsedRng, ray: &Ray, hit_record: &HitRecord) -> Scatter;
}

#[derive(new)]
pub struct Lambertian {
    attenuation: Color,
}
impl Material for Lambertian {
    fn scatter(&self, rng: &mut UsedRng, _ray: &Ray, hit_record: &HitRecord) -> Scatter {
        let mut scatter_direction = hit_record.normal + Vector::random_unit(rng);

        // Catch degenerate scatter directions and just make them normal
        if scatter_direction.near_zero() {
            scatter_direction = hit_record.normal;
        }

        Scatter {
            attenuation: self.attenuation,
            ray: Some(Ray::new(hit_record.point, scatter_direction)),
        }
    }
}

#[derive(new)]
pub struct Metal {
    /// The attenuation in the range [0, 1] for each color channel of the color that comes back from the scattered ray.
    attenuation: Color,
    /// Radius of the random deflection sphere added to the end of the reflected ray.
    ///
    /// Zero is is perfect reflection with no fuzziness.
    fuzz_factor: f64,
}
impl Material for Metal {
    fn scatter(&self, rng: &mut UsedRng, ray: &Ray, hit_record: &HitRecord) -> Scatter {
        let fuzz = clamp(self.fuzz_factor, 0., 1.);
        let reflected = ray.direction.reflect(hit_record.normal).normalize()
            + fuzz * Vector::random_within_unit_sphere(rng);

        Scatter {
            attenuation: self.attenuation,
            ray: Some(Ray::new(hit_record.point, reflected)),
        }
    }
}
