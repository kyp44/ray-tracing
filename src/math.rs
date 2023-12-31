use cgmath::{InnerSpace, Vector3, VectorSpace};
use derive_new::new;
use rand::{distributions::Uniform, prelude::Distribution, Rng};
use roots::find_roots_quadratic;
use std::ops::Range;

const NEAR_ZERO_THRESH: f64 = 1e-8;

pub type Point = cgmath::Point3<f64>;
pub type Vector = cgmath::Vector3<f64>;

pub trait VectorExt:
    Sized + VectorSpace<Scalar = f64> + InnerSpace + std::ops::Neg<Output = Self>
{
    /// A random vector, with each component chosen from a uniform distribution in the `range`.
    fn random<R: Rng>(rng: &mut R, range: Range<f64>) -> Self;

    fn random_unit_cube<R: Rng>(rng: &mut R) -> Self {
        Self::random(rng, 0.0..1.)
    }

    /// Returns a nonzero vector within the unit sphere.
    fn random_within_unit_sphere<R: Rng>(rng: &mut R) -> Self {
        loop {
            let v = Self::random(rng, -1.0..1.);
            if !v.near_zero() && v.magnitude2() < 1. {
                break v;
            }
        }
    }

    /// Returns a vector in the unit disk in the x-y plane.
    fn random_within_unit_disk<R: Rng>(rng: &mut R) -> Self;

    /// Returns a unit vector in a random direction.
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

    // This will have the same length as this vector
    fn reflect(&self, normal: Self) -> Self;

    // `eta_ratio` is the incident eta over the transmission eta.
    // Both this and the normal vector should be unit length.
    fn refract(&self, normal: Self, eta_ratio: f64) -> Self;
    fn average(vectors: impl Iterator<Item = Self>) -> Self;

    fn map<T>(&self, f: impl Fn(f64) -> T) -> Vector3<T>;
    fn all(&self, f: impl Fn(f64) -> bool) -> bool;

    fn near_zero(&self) -> bool;
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

    fn random_within_unit_disk<R: Rng>(rng: &mut R) -> Self {
        let between = Uniform::new(-1., 1.);
        loop {
            let v = Self::new(between.sample(rng), between.sample(rng), 0.);

            if v.magnitude2() < 1. {
                break v;
            }
        }
    }

    fn reflect(&self, normal: Self) -> Self {
        self - 2. * self.dot(normal) * normal
    }

    fn refract(&self, normal: Self, eta_ratio: f64) -> Self {
        let cos_theta = self.dot(-normal).min(1.);
        let r_perp = eta_ratio * (self + cos_theta * normal);
        let r_par = -(1. - r_perp.magnitude2()).abs().sqrt() * normal;

        r_perp + r_par
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

    fn all(&self, f: impl Fn(f64) -> bool) -> bool {
        f(self.x) && f(self.y) & f(self.z)
    }

    fn near_zero(&self) -> bool {
        self.all(|x| x.abs() < NEAR_ZERO_THRESH)
    }
}

#[derive(new)]
pub struct DirectionVectors {
    pub u: Vector,
    pub v: Vector,
}

#[derive(new)]
pub struct BasisVectors {
    pub u: Vector,
    pub v: Vector,
    pub w: Vector,
}

#[derive(new, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}
impl Ray {
    pub fn at(&self, t: f64) -> Point {
        self.origin + t * self.direction
    }
}

#[derive(Debug)]
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
