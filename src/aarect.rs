use bevy_math::vec3;
use crate::aabb::AABB;
use crate::material::Material;
use crate::obj::{HitResult, Hittable};
use crate::types::Ray;

pub struct XYRect<T: Material> {
    material: T,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    z: f32,
}

impl<T: Material> XYRect<T> {
    pub fn new(x0: f32, x1: f32, y0: f32, y1: f32, z: f32, material: T) -> Self {
        Self {
            material,
            x0,
            x1,
            y0,
            y1,
            z,
        }
    }
}

impl<T: Material> Hittable for XYRect<T> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let t = (self.z - ray.origin.z) / ray.direction.z;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let y = ray.origin.y + t * ray.direction.y;
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        let outward_normal = vec3(0.0, 0.0, 1.0);
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        let position = ray.at(t);
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        if !self.material.hack_solid(u, v, position) {
            return None;
        }

        Some(HitResult {
            position,
            normal,
            t,
            front_face,
            material: &self.material,
            u,
            v,
        })
    }

    fn bounding_box(&self) -> AABB {
        AABB::new(
            vec3(self.x0, self.y0, self.z - 0.0001),
            vec3(self.x1, self.y1, self.z + 0.0001),
        )
    }
}

pub struct XZRect<T: Material> {
    material: T,
    x0: f32,
    x1: f32,
    z0: f32,
    z1: f32,
    y: f32,
}

impl<T: Material> XZRect<T> {
    pub fn new(x0: f32, x1: f32, z0: f32, z1: f32, y: f32, material: T) -> Self {
        Self {
            material,
            x0,
            x1,
            z0,
            z1,
            y,
        }
    }
}

impl<T: Material> Hittable for XZRect<T> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let t = (self.y - ray.origin.y) / ray.direction.y;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let outward_normal = vec3(0.0, 1.0, 0.0);
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        let position = ray.at(t);
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        if !self.material.hack_solid(u, v, position) {
            return None;
        }

        Some(HitResult {
            position,
            normal,
            t,
            front_face,
            material: &self.material,
            u,
            v,
        })
    }

    fn bounding_box(&self) -> AABB {
        AABB::new(
            vec3(self.x0, self.y - 0.0001, self.z0),
            vec3(self.x1, self.y + 0.0001, self.z1),
        )
    }
}

pub struct YZRect<T: Material> {
    material: T,
    y0: f32,
    y1: f32,
    z0: f32,
    z1: f32,
    x: f32,
}

impl<T: Material> YZRect<T> {
    pub fn new(y0: f32, y1: f32, z0: f32, z1: f32, x: f32, material: T) -> Self {
        Self {
            material,
            y0,
            y1,
            z0,
            z1,
            x,
        }
    }
}

impl<T: Material> Hittable for YZRect<T> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let t = (self.x - ray.origin.x) / ray.direction.x;
        if t < t_min || t > t_max {
            return None;
        }
        let y = ray.origin.y + t * ray.direction.y;
        let z = ray.origin.z + t * ray.direction.z;
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let outward_normal = vec3(1.0, 0.0, 0.0);
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        let position = ray.at(t);
        let u = (y - self.y0) / (self.y1 - self.y0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        if !self.material.hack_solid(u, v, position) {
            return None;
        }

        Some(HitResult {
            position,
            normal,
            t,
            front_face,
            material: &self.material,
            u,
            v,
        })
    }

    fn bounding_box(&self) -> AABB {
        AABB::new(
            vec3(self.x - 0.0001, self.y0, self.z0),
            vec3(self.x + 0.0001, self.y1, self.z1),
        )
    }
}
