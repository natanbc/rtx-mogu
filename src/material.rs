use bevy_math::Vec4;
use rand::Rng;
use crate::obj::HitResult;
use crate::texture::{SolidColor, Texture};
use crate::types::{Color, Point3, Ray};
use crate::util;
use crate::util::{near_zero, random_in_unit_sphere, reflect, reflectance, refract, unit_vector};

pub trait Material {
    //Hack to implement transparency for textures
    fn hack_solid(&self, _: f32, _: f32, _: Point3) -> bool;

    fn emitted(&self, _: f32, _: f32, _: Point3) -> Color {
        Vec4::splat(0.0)
    }

    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<(Color, Ray)>;
}

#[derive(Copy, Clone)]
pub struct Lambertian<T: Texture> {
    albedo: T,
}

impl<T: Texture> Lambertian<T> {
    pub fn new(albedo: T) -> Self {
        Self {
            albedo,
        }
    }
}

impl Lambertian<SolidColor> {
    pub fn color(albedo: Color) -> Self {
        Self::new(SolidColor::new(albedo))
    }
}

impl<T: Texture> Material for Lambertian<T> {
    fn hack_solid(&self, u: f32, v: f32, p: Point3) -> bool {
        self.albedo.hack_solid(u, v, p)
    }

    fn scatter(&self, _: &Ray, hit: &HitResult) -> Option<(Color, Ray)> {
        let mut scatter_direction = hit.normal + util::random_unit_vector();
        if near_zero(scatter_direction) {
            scatter_direction = hit.normal;
        }
        Some((self.albedo.value(hit.u, hit.v, hit.position), Ray::new(hit.position, scatter_direction)))
    }
}

#[derive(Copy, Clone)]
pub struct Metal<T: Texture> {
    albedo: T,
    fuzz: f32,
}

impl<T: Texture> Metal<T> {
    pub fn new(albedo: T, fuzz: f32) -> Self {
        Self {
            albedo,
            fuzz,
        }
    }
}

impl Metal<SolidColor> {
    pub fn color(albedo: Color, fuzz: f32) -> Self {
        Self::new(SolidColor::new(albedo), fuzz)
    }
}

impl<T: Texture> Material for Metal<T> {
    fn hack_solid(&self, u: f32, v: f32, p: Point3) -> bool {
        self.albedo.hack_solid(u, v, p)
    }

    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<(Color, Ray)> {
        let reflected = reflect(unit_vector(ray.direction), hit.normal);
        let scattered = Ray::new(hit.position, reflected + self.fuzz * random_in_unit_sphere());
        if scattered.direction.dot(hit.normal) > 0.0 {
            Some((self.albedo.value(hit.u, hit.v, hit.position), scattered))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct Dielectric<T: Texture> {
    texture: T,
    ir: f32,
}

impl<T: Texture> Dielectric<T> {
    pub fn new(texture: T, index_of_refraction: f32) -> Self {
        Self {
            texture,
            ir: index_of_refraction,
        }
    }
}

impl<T: Texture> Material for Dielectric<T> {
    fn hack_solid(&self, u: f32, v: f32, p: Point3) -> bool {
        self.texture.hack_solid(u, v, p)
    }

    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<(Color, Ray)> {
        let refraction_ratio = if hit.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_dir = unit_vector(ray.direction);

        let cos_theta = (-unit_dir).dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

        let direction = if refraction_ratio * sin_theta > 1.0 || reflectance(cos_theta, refraction_ratio) > rand::thread_rng().gen::<f32>() {
            reflect(unit_dir, hit.normal)
        } else {
            refract(unit_dir, hit.normal, refraction_ratio)
        };

        Some((self.texture.value(hit.u, hit.v, hit.position), Ray::new(hit.position, direction)))
    }
}

#[derive(Copy, Clone)]
pub struct DiffuseLight<T: Texture> {
    texture: T,
}

impl<T: Texture> DiffuseLight<T> {
    pub fn new(texture: T) -> Self {
        Self {
            texture,
        }
    }
}

impl DiffuseLight<SolidColor> {
    pub fn color(color: Color) -> Self {
        Self::new(SolidColor::new(color))
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn hack_solid(&self, _: f32, _: f32, _: Point3) -> bool {
        true
    }

    fn emitted(&self, u: f32, v: f32, p: Point3) -> Color {
        self.texture.value(u, v, p)
    }

    fn scatter(&self, _: &Ray, _: &HitResult) -> Option<(Color, Ray)> {
        None
    }
}
