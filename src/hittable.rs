use crate::{
    material::Material,
    math::{Parabola, ParabolaRoots, Point, Ray, Vector},
};
use cgmath::InnerSpace;
use derive_new::new;
use std::ops::RangeInclusive;

#[derive(Debug)]
pub struct HitRecord<'a> {
    pub point: Point,
    // This normal always faces against the ray.
    pub normal: Vector,
    pub t: f64,
    // The front face was hit.
    pub front_face: bool,
    pub material: &'a dyn Material,
}
impl<'a> HitRecord<'a> {
    fn new(material: &'a dyn Material, ray: &Ray, t: f64, outward_normal: Vector) -> Self {
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
            material,
        }
    }
}

pub trait Hittable: Sync {
    fn hit(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitRecord>;
}

#[derive(new)]
pub struct HittableList<'a> {
    list: &'a [&'a dyn Hittable],
}
impl Hittable for HittableList<'_> {
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
    material: Box<dyn Material + Sync>,
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
            ParabolaRoots::One(r) => Some(vec![r]),
            ParabolaRoots::Two(r1, r2) => Some(vec![r1, r2]),
        }
        .and_then(|rs| {
            for t in rs {
                if t_range.contains(&t) {
                    return Some(HitRecord::new(
                        self.material.as_ref(),
                        ray,
                        t,
                        (ray.at(t) - self.center) / self.radius,
                    ));
                }
            }
            None
        })
    }
}
