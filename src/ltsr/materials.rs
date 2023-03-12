use crate::ltsr::{
    near_zero, random_in_hemisphere, random_in_unit_sphere, reflect, Color, HitData, Ray,
};

/// Anything that is considered a material should implement this trait
pub trait Material {
    fn scatter(&self, ray: &Ray, data: &HitData) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, data: &HitData) -> Option<(Color, Ray)> {
        // Scatter a new ray in a random direction, but based
        // on the Normal of the object that we have just hit
        let mut scatter_direction = data.hit_point + random_in_hemisphere(data.normal);

        // Catch eventual degenerate directions and correct them
        // (this could happen if the normal and the random direction are completely opposite)
        if near_zero(&scatter_direction) {
            scatter_direction = data.normal;
        }
        let new_ray = Ray::new(data.hit_point, scatter_direction - data.hit_point);

        Some((self.albedo, new_ray.clone()))
    }
}

pub struct Metallic {
    pub albedo: Color,
    pub roughness: f32,
}

impl Metallic {
    pub fn new(albedo: Color, roughness: f32) -> Self {
        Self { albedo, roughness }
    }
}

impl Material for Metallic {
    fn scatter(&self, ray_in: &Ray, data: &HitData) -> Option<(Color, Ray)> {
        // Scatter a new ray in a based on the Normal of the object that we have just hit
        let reflected_direction = reflect(ray_in.direction, data.normal).normalize();

        // Mimic the metallic imperfections by moving the reflected ray a bit
        // let roughness_perturbation = self.roughness * random_in_hemisphere(data.normal);
        let roughness_perturbation = self.roughness * random_in_unit_sphere();

        let new_ray = Ray::new(data.hit_point, reflected_direction + roughness_perturbation);

        // If the new ray is not pointint outside the object, don't return it
        if new_ray.direction.dot(data.normal) > 0.0 {
            Some((self.albedo, new_ray.clone()))
        } else {
            None
        }
    }
}
