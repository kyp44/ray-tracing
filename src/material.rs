use crate::{
    hittable::HitRecord,
    image::Color,
    math::{Ray, Vector, VectorExt},
    UsedRng,
};
use cgmath::InnerSpace;
use derive_new::new;

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
    attenuation: Color,
}
impl Material for Metal {
    fn scatter(&self, _rng: &mut UsedRng, ray: &Ray, hit_record: &HitRecord) -> Scatter {
        let reflected = ray.direction.reflect(hit_record.normal).normalize();

        Scatter {
            attenuation: self.attenuation,
            ray: Some(Ray::new(hit_record.point, reflected)),
        }
    }
}
