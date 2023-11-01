use cgmath::{InnerSpace, Vector3, VectorSpace};
use derive_new::new;
use rand::{distributions::Uniform, prelude::Distribution, Rng};
use roots::find_roots_quadratic;
use std::ops::Range;

pub type Point = cgmath::Point3<f64>;
pub type Vector = cgmath::Vector3<f64>;

pub trait VectorExt:
    Sized + VectorSpace<Scalar = f64> + InnerSpace + std::ops::Neg<Output = Self>
{
    fn random<R: Rng>(rng: &mut R, range: Range<f64>) -> Self;

    fn random_unit_cube<R: Rng>(rng: &mut R) -> Self {
        Self::random(rng, 0.0..1.)
    }

    // This is necessary so that the unit sphere has a uniform distribution
    fn random_within_unit_sphere<R: Rng>(rng: &mut R) -> Self {
        loop {
            let v = Self::random(rng, -1.0..1.);
            if v.magnitude2() < 1. {
                break v;
            }
        }
    }

    fn random_unit<R: Rng>(rng: &mut R) -> Self {
        Self::random_within_unit_sphere(rng).normalize()
    }

    fn random_on_hemisphere<R: Rng>(rng: &mut R, normal: Self) -> Self {
        let unit = Self::random_unit(rng);

        if unit.dot(normal) > 0.0 {
            unit
        } else {
            -unit
        }
    }

    fn average(vectors: impl Iterator<Item = Self>) -> Self;
    fn map<T>(&self, f: impl Fn(f64) -> T) -> Vector3<T>;
}
impl VectorExt for Vector {
    fn random<R: Rng>(rng: &mut R, range: Range<f64>) -> Self {
        let between = Uniform::new(range.start, range.end);
        Self::new(
            between.sample(rng),
            between.sample(rng),
            between.sample(rng),
        )
    }

    fn average(vectors: impl Iterator<Item = Self>) -> Self {
        let mut n: u32 = 1;
        let sum = vectors
            .reduce(|a, b| {
                n += 1;
                a + b
            })
            .expect("The vector iterator cannot be empty");

        sum / f64::from(n)
    }

    fn map<T>(&self, f: impl Fn(f64) -> T) -> Vector3<T> {
        Vector3::new(f(self.x), f(self.y), f(self.z))
    }
}

#[derive(new)]
pub struct DirectionVectors {
    pub u: Vector,
    pub v: Vector,
}

#[derive(new)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}
impl Ray {
    pub fn at(&self, t: f64) -> Point {
        self.origin + t * self.direction
    }
}

pub enum ParabolaRoots {
    None,
    One(f64),
    // In order of absolute value
    Two(f64, f64),
}

#[derive(new)]
pub struct Parabola {
    a: f64,
    b: f64,
    c: f64,
}
impl Parabola {
    pub fn roots(&self) -> ParabolaRoots {
        use roots::Roots;

        match find_roots_quadratic(self.a, self.b, self.c) {
            Roots::One(r) => ParabolaRoots::One(r[0]),
            Roots::Two(r) => {
                let roots = std::cmp::minmax_by(r[0], r[1], |x, y| x.partial_cmp(y).unwrap());

                ParabolaRoots::Two(roots[0], roots[1])
            }
            _ => ParabolaRoots::None,
        }
    }
}
