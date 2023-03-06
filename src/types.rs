use bevy_math::{Vec3, Vec4};

pub type Point3 = Vec3;

pub type Color = Vec4;

#[derive(Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
        }
    }

    pub fn at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }
}
