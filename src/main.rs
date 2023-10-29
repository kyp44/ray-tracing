use cgmath::{InnerSpace, MetricSpace, Point3, Vector3, VectorSpace, Zero};
use derive_more::{Add, From, Sub, Sum};
use derive_new::new;
use easy_cast::ConvFloat;
use indicatif::ProgressBar;
use itertools::iproduct;
use num::{rational::Ratio, ToPrimitive};

const IMAGE_WIDTH: u32 = 400;
const VIEWPORT_HEIGHT: f64 = 2.0;
const FOCAL_LENGTH: f64 = 1.0;
const CAMERA_CENTER: Point = Point::new(0., 0., 0.);
const MAX_COLOR: u8 = 255;

#[derive(Debug, new)]
struct Size<T> {
    width: T,
    height: T,
}
impl<T> Size<T>
where
    T: std::ops::Mul<T, Output = T> + num::Integer + ToPrimitive + num::bigint::ToBigInt + Copy,
{
    fn len(&self) -> T {
        self.width * self.height
    }

    fn aspect_ratio(&self) -> f64 {
        Ratio::new(self.width, self.height).to_f64().unwrap()
    }
}

type Channel = f64;

#[derive(Clone, Copy, Add, Sub, Sum, From)]
struct Color(Vector);
impl Color {
    fn new(red: f64, green: f64, blue: f64) -> Self {
        Self(Vector::new(red, green, blue))
    }
}
impl std::ops::Mul<Channel> for Color {
    type Output = Self;

    fn mul(self, rhs: Channel) -> Self::Output {
        Self::from(rhs * self.0)
    }
}
impl std::ops::Div<Channel> for Color {
    type Output = Self;

    fn div(self, rhs: Channel) -> Self::Output {
        Self::from(self.0 / rhs)
    }
}
impl std::ops::Rem<Channel> for Color {
    type Output = Self;

    fn rem(self, rhs: Channel) -> Self::Output {
        Self::from(self.0 % rhs)
    }
}
impl Zero for Color {
    fn zero() -> Self {
        Self::from(Vector::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.x.is_zero() && self.0.y.is_zero() && self.0.z.is_zero()
    }
}
impl VectorSpace for Color {
    type Scalar = f64;
}
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn convert_channel(v: Channel) -> u8 {
            u8::conv_nearest(Channel::from(MAX_COLOR) * v)
        }

        write!(
            f,
            "{} {} {}",
            convert_channel(self.0.x),
            convert_channel(self.0.y),
            convert_channel(self.0.z),
        )
    }
}

type Point = Point3<f64>;
type Vector = Vector3<f64>;

#[derive(new)]
struct DirectionVectors {
    u: Vector,
    v: Vector,
}

#[derive(new)]
struct Ray {
    origin: Point,
    direction: Vector,
}
impl Ray {
    fn at(&self, t: f64) -> Point {
        self.origin + t * self.direction
    }
}

#[derive(new)]
struct Parabola {
    a: f64,
    b: f64,
    c: f64,
}
impl Parabola {
    fn discriminant(&self) -> f64 {
        self.b.powi(2) - 4. * self.a * self.c
    }
}

fn main() {
    // Calculate the image size
    let aspect_ratio = Ratio::new(16u32, 9);
    let image_size = Size::new(
        IMAGE_WIDTH,
        (Ratio::from(IMAGE_WIDTH) / aspect_ratio).to_integer(),
    );

    // Calculate the viewport size based on the image size
    let viewport_size = Size::new(image_size.aspect_ratio() * VIEWPORT_HEIGHT, VIEWPORT_HEIGHT);

    // Set the viewport edge vectors
    let viewport_edge_vectors = DirectionVectors::new(
        Vector::new(viewport_size.width, 0., 0.),
        Vector::new(0., -viewport_size.height, 0.),
    );

    // Set the pixel-to-pixel vectors
    let pixel_delta_vectors = DirectionVectors::new(
        viewport_edge_vectors.u / image_size.width.into(),
        viewport_edge_vectors.v / image_size.height.into(),
    );

    // Calculate the location of the upper left of the viewport
    let viewport_upper_left = CAMERA_CENTER
        - Vector::new(0., 0., FOCAL_LENGTH)
        - viewport_edge_vectors.u / 2.
        - viewport_edge_vectors.v / 2.;

    // Calculate upper left pixel location
    let pixel_upper_left =
        viewport_upper_left + 0.5 * (pixel_delta_vectors.u + pixel_delta_vectors.v);

    // Print header: P3 format, image size, and max color value
    println!(
        "P3\n{} {}\n{MAX_COLOR}",
        image_size.width, image_size.height
    );

    fn hit_sphere(center: &Point, radius: f64, ray: &Ray) -> bool {
        Parabola::new(
            ray.direction.magnitude2(),
            2. * ray.direction.dot(ray.origin - center),
            (ray.origin - center).magnitude2() - radius.powi(2),
        )
        .discriminant()
            >= 0.
    }

    fn ray_color(ray: &Ray) -> Color {
        if hit_sphere(&Point::new(0., 0., -1.), 0.5, ray) {
            Color::new(1., 0., 0.)
        } else {
            let unit = ray.direction.normalize();
            Color::new(1., 1., 1.).lerp(Color::new(0.5, 0.7, 1.), 0.5 * (unit.y + 1.))
        }
    }

    // Render the scene
    let bar = ProgressBar::new(image_size.len().into());
    for (y, x) in iproduct!(0..image_size.height, 0..image_size.width) {
        bar.inc(1);

        // Project the ray from the camera through the pixel
        let pixel_center = pixel_upper_left
            + f64::from(x) * pixel_delta_vectors.u
            + f64::from(y) * pixel_delta_vectors.v;
        let ray = Ray::new(CAMERA_CENTER, pixel_center - CAMERA_CENTER);

        println!("{}", ray_color(&ray));
    }
    bar.finish_and_clear();
}
