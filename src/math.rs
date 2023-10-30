use derive_new::new;
use roots::find_roots_quadratic;

pub type Point = cgmath::Point3<f64>;
pub type Vector = cgmath::Vector3<f64>;

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
