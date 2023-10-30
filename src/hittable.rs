use crate::{
    math::{Parabola, ParabolaRoots, Point, Ray, Vector},
    Color,
};
use cgmath::InnerSpace;
use derive_new::new;
use std::ops::RangeInclusive;

struct HitRecord {
    point: Point,
    // This normal always faces against the ray
    normal: Vector,
    t: f64,
    // The front face was hit
    front_face: bool,
}
impl HitRecord {
    fn new(point: Point, t: f64, ray: &Ray, outward_normal: Vector) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.;
    }
}

trait Hittable {
    fn hit(&self, ray: &Ray, t_range: RangeInclusive<f64>) -> Option<HitRecord>;
}

#[derive(new)]
pub struct Sphere {
    center: Point,
    radius: f64,
}
impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: RangeInclusive<f64>) -> Option<HitRecord> {
        let oc = ray.origin - self.center;

        match Parabola::new(
            ray.direction.magnitude2(),
            2. * oc.dot(ray.direction),
            oc.magnitude2() - self.radius.powi(2),
        )
        .roots()
        {
            ParabolaRoots::None => None,
            ParabolaRoots::One(r) => Some(r),
            ParabolaRoots::Two(r, _) => Some(r),
        }
        .and_then(|t| {
            if t_range.contains(&t) {
                Some(HitRecord {
                    point: ray.at(t),
                    normal: (ray.at(t) - self.center) / self.radius,
                    t,
                })
            } else {
                None
            }
        })
    }
}
impl Sphere {
    // TODO: delete me!
    pub fn hit_color(&self, ray: &Ray) -> Option<Color> {
        let oc = ray.origin - self.center;

        match Parabola::new(
            ray.direction.magnitude2(),
            2. * oc.dot(ray.direction),
            oc.magnitude2() - self.radius.powi(2),
        )
        .roots()
        {
            ParabolaRoots::None => None,
            ParabolaRoots::One(r) => Some(r),
            ParabolaRoots::Two(r, _) => Some(r),
        }
        .map(|t| {
            let norm = (ray.at(t) - self.center).normalize();

            (Color::from(norm) + Color::new(1., 1., 1.)) * 0.5
        })
    }
}
