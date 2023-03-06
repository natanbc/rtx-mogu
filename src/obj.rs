use std::marker::PhantomData;
use std::sync::Arc;
use bevy_math::{Vec3, vec3};
use crate::aabb::AABB;
use crate::material::Material;
use crate::types::{Point3, Ray};

pub struct HitResult<'a> {
    pub position: Point3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub material: &'a dyn Material,
    pub u: f32,
    pub v: f32,
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult>;

    fn bounding_box(&self) -> AABB;
}

pub struct HittableList {
    objs: Vec<Arc<dyn Hittable + Send>>,
}

unsafe impl Send for HittableList {}
unsafe impl Sync for HittableList {}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objs: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: impl Hittable + Send + 'static) {
        self.objs.push(Arc::new(obj));
    }

    pub fn clear(&mut self) {
        self.objs.clear();
    }

    pub fn into_vec(self) -> Vec<Arc<dyn Hittable + Send>> {
        self.objs
    }

    pub fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let mut best = None;
        let mut closest = t_max;
        for obj in self.objs.iter() {
            let res = obj.hit(&ray, t_min, closest);
            if let Some(res) = res {
                closest = res.t;
                best = Some(res);
            }
        }
        best
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        if self.objs.is_empty() {
            return None;
        }

        let mut iter = self.objs.iter();
        let mut bbox = iter.next().unwrap().bounding_box();
        for obj in iter {
            bbox = AABB::surrounding_box(bbox, obj.bounding_box());
        }

        Some(bbox)
    }
}

pub struct Sphere<T: Material> {
    center: Point3,
    radius: f32,
    material: T,
}

impl<T: Material> Sphere<T> {
    pub fn new(center: Point3, radius: f32, material: T) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl<T: Material> Hittable for Sphere<T> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_disc = discriminant.sqrt();

        let mut root = (-half_b - sqrt_disc) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrt_disc) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let t = root;
        let p = ray.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + std::f32::consts::PI;

        let u = phi / (2.0 * std::f32::consts::PI);
        let v = theta / std::f32::consts::PI;

        if !self.material.hack_solid(u, v, p) {
            return None;
        }

        Some(HitResult {
            position: p,
            normal,
            t,
            front_face,
            material: &self.material,
            u,
            v,
        })
    }

    fn bounding_box(&self) -> AABB {
        let rv = Vec3::splat(self.radius);
        AABB::new(self.center - rv, self.center + rv)
    }
}

pub trait RotateVec3 {
    fn rotate(v: Vec3, sin_theta: f32, cos_theta: f32) -> Vec3;
}

pub struct RotateVec3X;
impl RotateVec3 for RotateVec3X {
    fn rotate(v: Vec3, sin_theta: f32, cos_theta: f32) -> Vec3 {
        let y = v.y * cos_theta  - v.z * sin_theta;
        let z = v.y * sin_theta + v.z * cos_theta;
        vec3(v.x, y, z)
    }
}

pub struct RotateVec3Y;
impl RotateVec3 for RotateVec3Y {
    fn rotate(v: Vec3, sin_theta: f32, cos_theta: f32) -> Vec3 {
        let x = v.x * cos_theta + v.z * sin_theta;
        let z = -v.x * sin_theta + v.z * cos_theta;
        vec3(x, v.y, z)
    }
}

pub struct RotateVec3Z;
impl RotateVec3 for RotateVec3Z {
    fn rotate(v: Vec3, sin_theta: f32, cos_theta: f32) -> Vec3 {
        let x = v.x * cos_theta - v.y * sin_theta;
        let y = v.x * sin_theta + v.y * cos_theta;
        vec3(x, y, v.z)
    }
}

pub struct Rotate<O: Hittable, R: RotateVec3> {
    r: PhantomData<R>,
    obj: O,
    bbox: AABB,
    sin_theta: f32,
    cos_theta: f32,
}

impl<O: Hittable, R: RotateVec3> Rotate<O, R> {
    pub fn new(obj: O, theta: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let orig_bbox = obj.bounding_box();
        let mut min = Vec3::splat(f32::INFINITY).to_array();
        let mut max = Vec3::splat(f32::NEG_INFINITY).to_array();

        for i in [0.0f32, 1.0f32] {
            for j in [0.0f32, 1.0f32] {
                for k in [0.0f32, 1.0f32] {
                    let x = i * orig_bbox.max.x + (1.0 - i) * orig_bbox.min.x;
                    let y = j * orig_bbox.max.y + (1.0 - j) * orig_bbox.min.y;
                    let z = k * orig_bbox.max.z + (1.0 - k) * orig_bbox.min.z;

                    let tester = R::rotate(vec3(x, y, z), sin_theta, cos_theta).to_array();
                    for c in 0..3 {
                        min[c] = min[c].min(tester[c]);
                        max[c] = max[c].max(tester[c]);
                    }
                }
            }
        }

        Self {
            r: PhantomData,
            obj,
            bbox: AABB::new(Vec3::from_array(min), Vec3::from_array(max)),
            sin_theta,
            cos_theta,
        }
    }
}

impl<O: Hittable, R: RotateVec3> Hittable for Rotate<O, R> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        //-self.sin_theta because sin(-X) = -sin(X), cos(-X) = cos(X)
        let origin = R::rotate(ray.origin, -self.sin_theta, self.cos_theta);
        let direction = R::rotate(ray.direction, -self.sin_theta, self.cos_theta);

        let rotated_ray = Ray::new(origin, direction);
        let mut res = self.obj.hit(&rotated_ray, t_min, t_max)?;

        let p = R::rotate(res.position, self.sin_theta, self.cos_theta);
        let normal = R::rotate(res.normal, self.sin_theta, self.cos_theta);

        let front_face = rotated_ray.direction.dot(normal) < 0.0;
        let normal = if front_face {
            normal
        } else {
            -normal
        };

        res.position = p;
        res.front_face = front_face;
        res.normal = normal;

        Some(res)
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}

pub type RotateX<O> = Rotate<O, RotateVec3X>;
pub type RotateY<O> = Rotate<O, RotateVec3Y>;
pub type RotateZ<O> = Rotate<O, RotateVec3Z>;

pub struct Translate<O: Hittable> {
    obj: O,
    translation: Vec3,
    bbox: AABB,
}

impl<O: Hittable> Translate<O> {
    pub fn new(obj: O, translation: Vec3) -> Self {
        let bbox = obj.bounding_box();
        let bbox = AABB::new(bbox.min + translation, bbox.max + translation);
        Self {
            obj,
            translation,
            bbox,
        }
    }
}

impl<O: Hittable> Hittable for Translate<O> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let moved_ray = Ray::new(ray.origin - self.translation, ray.direction);
        let mut res = self.obj.hit(&moved_ray, t_min, t_max)?;

        let front_face = moved_ray.direction.dot(res.normal) < 0.0;
        let normal = if front_face {
            res.normal
        } else {
            -res.normal
        };

        res.position += self.translation;
        res.front_face = front_face;
        res.normal = normal;

        Some(res)
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
