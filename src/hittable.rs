use crate::math::{Parabola, ParabolaRoots, Point, Ray, Vector};
use cgmath::InnerSpace;
use derive_new::new;
use std::ops::RangeInclusive;

pub struct HitRecord {
    pub point: Point,
    // This normal always faces against the ray.
    pub normal: Vector,
    pub t: f64,
    // The front face was hit.
    pub front_face: bool,
}
impl HitRecord {
    fn new(ray: &Ray, t: f64, outward_normal: Vector) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.;

        HitRecord {
            point: ray.at(t),
            normal: if front_face {
                outward_normal
            } else {
                -outward_normal
            },
            t,
            front_face,
        }
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitRecord>;
}

#[derive(new)]
pub struct HittableList {
    list: Box<[Box<dyn Hittable>]>,
}
impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitRecord> {
        self.list.iter().fold(None, |current, next| {
            let next = next.hit(
                ray,
                &RangeInclusive::new(
                    *t_range.start(),
                    current.as_ref().map(|hr| hr.t).unwrap_or(*t_range.end()),
                ),
            );

            next.or(current)
        })
    }
}

#[derive(new)]
pub struct Sphere {
    center: Point,
    radius: f64,
}
impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitRecord> {
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
                Some(HitRecord::new(
                    ray,
                    t,
                    (ray.at(t) - self.center) / self.radius,
                ))
            } else {
                None
            }
        })
    }
}
