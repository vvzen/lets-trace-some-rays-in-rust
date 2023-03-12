use std::sync::Arc;

use glam::Vec3;
use rand::distributions::{Distribution, Uniform};

pub mod materials;

use materials::Material;

type Color = Vec3;

/// Holds information on a raycast hit event
pub struct HitData {
    hit_point: Vec3,
    normal: Vec3,
    material: Arc<dyn Material>,
    t: f32,
}

/// Anything that can be hit should implement this trait!
pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitData>;
}

// Scene and Geometry
// ----------------------------------------------------------------------------

pub struct Scene {
    pub elements: Vec<Box<dyn Hittable>>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { elements: vec![] }
    }

    pub fn add_hittable(self: &mut Self, hittable: Box<dyn Hittable>) {
        self.elements.push(hittable);
    }
}

impl Hittable for Scene {
    fn hit(self: &Self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitData> {
        let mut closest_hit: Option<HitData> = None;
        let mut closest_so_far = t_max;

        // Iterate through all the elements in the scene
        for element in self.elements.iter() {
            match element.hit(ray, t_min, closest_so_far) {
                // We have a hit!
                Some(hit_data) => {
                    closest_so_far = hit_data.t;
                    closest_hit = Some(hit_data);
                }
                // Nothing to do, since the ray didn't hit anything
                None => {}
            }
        }

        closest_hit
    }
}

fn get_face_normal(ray: &Ray, outward_normal: Vec3) -> (Vec3, bool) {
    let is_front_face = ray.direction.dot(outward_normal) < 0.0;

    let normal;
    if is_front_face {
        normal = outward_normal;
    } else {
        normal = (-1.0) * outward_normal;
    }

    (normal, is_front_face)
}

pub struct Sphere {
    pub radius: f32,
    pub center: Vec3,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(radius: f32, center: Vec3, material: Arc<dyn Material>) -> Self {
        Sphere {
            radius,
            center,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(self: &Self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitData> {
        // The quadratic polynomial ax^2 + bx + c has discriminant b^2 - 4ac
        // See https://en.wikipedia.org/wiki/Discriminant

        let center_to_origin = ray.origin - self.center;

        let a = ray.direction.length_squared();
        let half_b = center_to_origin.dot(ray.direction);
        let c = center_to_origin.length_squared() - self.radius.powi(2);

        let discriminant = half_b.powi(2) - (a * c);

        // No hit!
        if discriminant < 0.0 {
            return None;
        }

        let discriminant_squared = discriminant.sqrt();

        // Quadratic formula: -b ± sqrt(b^2 - 4ac)
        // The above^ can have 2 solutions (because of the ±)
        // the first_solution is (-half_b + discriminant_squared) / a
        // the second_solution is (-half_b - discriminant_squared) / a
        // The smallest solution will be the closest to the ray origin

        // Find the nearest 't' that lies in the acceptable range ([t_min, t_max])
        let mut t = (-half_b - discriminant_squared) / a;

        if t < t_min || t_max < t {
            t = (-half_b + discriminant_squared) / a;
            if t < t_min || t_max < t {
                return None;
            }
        }

        let hit_point = ray.point_at_parameter(t);
        let outward_normal = (hit_point - self.center) / self.radius;

        let (normal, is_front_face) = get_face_normal(ray, outward_normal);

        Some(HitData {
            t,
            hit_point,
            normal,
            material: self.material.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray { origin, direction }
    }

    fn point_at_parameter(self: &Self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}

pub struct Camera {
    position: Vec3,
    right: Vec3,
    up: Vec3,
    back: Vec3,
    focal_length: f32,
    lower_left_corner: Vec3,
}

impl Camera {
    pub fn new(focal_length: f32, viewport_width: f32, viewport_height: f32) -> Self {
        let origin = Vec3::new(0.0, 0.0, 0.0);

        let right = Vec3::new(viewport_width, 0.0, 0.0);
        let up = Vec3::new(0.0, viewport_height, 0.0);
        let back = Vec3::new(0.0, 0.0, focal_length);

        let lower_left_corner = origin - (right / 2.0) - (up / 2.0) - back;
        eprintln!("Camera lower left corner: {lower_left_corner}");

        Camera {
            position: origin,
            right,
            up,
            back,
            lower_left_corner,
            focal_length,
        }
    }

    pub fn get_ray_at_coords(self: &Self, u: f32, v: f32) -> Ray {
        let camera_direction =
            self.lower_left_corner + u * self.right + v * self.up - self.position;

        Ray::new(self.position, camera_direction)
    }
}

/// Given a Ray and a Scene of objects, return the color
/// resulting with the Ray intersecting the Scene
pub fn ray_color(ray: &Ray, scene: &Scene, max_depth: i32) -> Color {
    // Offset the min a bit from 0 to avoid self-intersections caused
    // by rounding floating point conversions
    let t_min = 0.001;
    let t_max = f32::INFINITY;

    // We've exceeded the maximum amount of bounces
    // for the current object: return a black shadow!
    if max_depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    match scene.hit(ray, t_min, t_max) {
        Some(object) => {
            let new_max_depth = match max_depth.checked_sub(1) {
                Some(n) => n,
                None => {
                    panic!("Integer underflow occurred!");
                }
            };

            // Scatter the ray in the material of the object hit
            let scatter_result = object.material.scatter(&ray, &object);
            match scatter_result {
                Some((color, new_ray)) => {
                    // Shoot more rays to simulate ray bounces after scattering
                    // in the current material
                    let mut pixel_color = color.clone();
                    pixel_color *= ray_color(&new_ray, &scene, new_max_depth);

                    return pixel_color.clone();
                }
                None => {
                    // No more scattering, the ray was absorbed: turn it darker
                    return Color::new(0.0, 0.0, 0.0);
                }
            }
        }
        None => {}
    }

    // If we got here, it means that our ray didn't hit anything
    // Let's draw our background!
    get_background_color(ray)
}

fn get_background_color(ray: &Ray) -> Color {
    let unit_direction = ray.direction.normalize();

    // Two colors of our background
    let white = Vec3::new(1.0, 1.0, 1.0);
    let blue = Vec3::new(0.5, 0.7, 1.0);

    // Perform the lerp:
    let t = 0.5 * (unit_direction.y + 1.0);
    let color = white.lerp(blue, t);

    Color::new(color.x, color.y, color.z)
}

// Utility functions
// ----------------------------------------------------------------------------

/// Linear remap a value in one range into another range (no clamping)
pub fn fit_range(x: f32, imin: f32, imax: f32, omin: f32, omax: f32) -> f32 {
    (omax - omin) * (x - imin) / (imax - imin) + omin
}

/// Generate a random point in a unit sphere
fn random_in_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();

    // TODO: Investigate using a lazy_static for this
    let range = Uniform::from(-1.0..1.0);

    loop {
        let x: f32 = range.sample(&mut rng);
        let y: f32 = range.sample(&mut rng);
        let z: f32 = range.sample(&mut rng);

        let p = Vec3::new(x, y, z);

        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

/// Generate a random point in a unit sphere
/// but in the same hemisphere as a Normal vector
fn random_in_hemisphere(normal: Vec3) -> Vec3 {
    let in_unit_sphere = random_in_unit_sphere();

    if Vec3::dot(in_unit_sphere, normal) > 0.0 {
        return in_unit_sphere;
    } else {
        return -in_unit_sphere;
    }
}

/// Returns whether or not the given ``vec`` is close enough to zero
/// on this machine (uses f32::EPSILON as the 'clone enough' measure).
fn near_zero(vec: &Vec3) -> bool {
    vec.x.abs() <= f32::EPSILON && vec.y.abs() <= f32::EPSILON.abs() && vec.z <= f32::EPSILON
}

/// Given a vector ``vec`` and a normalized vector ``normal``,
/// return back ``vec`` reflected against ``normal``.
fn reflect(vec: Vec3, normal: Vec3) -> Vec3 {
    vec - 2.0 * vec.dot(normal) * normal
}
