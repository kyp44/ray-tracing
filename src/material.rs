use crate::{
    hittable::HitRecord,
    image::Color,
    math::{Ray, Vector, VectorExt},
    UsedRng,
};
use cgmath::InnerSpace;
use derive_new::new;
use num::clamp;
use rand::Rng;

pub struct Scatter {
    pub attenuation: Color,
    pub ray: Option<Ray>,
}

pub trait Material: std::fmt::Debug {
    fn scatter(&self, rng: &mut UsedRng, ray: &Ray, hit_record: &HitRecord) -> Scatter;
}

#[derive(new, Debug)]
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

#[derive(new, Debug)]
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

#[derive(new, Debug)]
pub struct Dielectric {
    index_of_refraction: f64,
}
impl Dielectric {
    /// The Schlick polynomial approximation for reflectance.
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = (1. - ref_idx) / (1. + ref_idx);
        let r0 = r0 * r0;
        r0 + (1. - r0) * ((1. - cosine).powi(5))
    }
}
impl Material for Dielectric {
    fn scatter(&self, rng: &mut UsedRng, ray: &Ray, hit_record: &HitRecord) -> Scatter {
        let eta_ratio = if hit_record.front_face {
            1. / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let unit_direction = ray.direction.normalize();
        let cos_theta = unit_direction.dot(-hit_record.normal).min(1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();
        let cannot_refract = eta_ratio * sin_theta > 1.0;

        let scatter_direction =
            if cannot_refract || Self::reflectance(cos_theta, eta_ratio) > rng.gen() {
                // Do not refract and completely reflect instead
                unit_direction.reflect(hit_record.normal)
            } else {
                // Refract
                unit_direction.refract(hit_record.normal, eta_ratio)
            };

        Scatter {
            attenuation: Color::new(1., 1., 1.),
            ray: Some(Ray::new(hit_record.point, scatter_direction)),
        }
    }
}
