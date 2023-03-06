use crate::types::{Point3, Ray};

#[derive(Copy, Clone)]
pub struct AABB {
    pub min: Point3,
    pub max: Point3,
}

impl AABB {
    pub fn new(min: Point3, max: Point3) -> Self {
        Self {
            min,
            max,
        }
    }

    pub fn surrounding_box(a: Self, b: Self) -> Self {
        let min = a.min.min(b.min);
        let max = a.max.max(b.max);
        Self::new(min, max)
    }

    pub fn hit(&self, ray: &Ray, mut t_min: f32, mut t_max: f32) -> bool {
        let min = self.min.to_array();
        let max = self.max.to_array();
        let origin = ray.origin.to_array();
        let direction = ray.direction.to_array();

        for i in 0..3 {
            let inv_d = 1.0 / direction[i];
            let mut t0 = (min[i] - origin[i]) * inv_d;
            let mut t1 = (max[i] - origin[i]) * inv_d;
            if inv_d < 0.0 {
                (t0, t1) = (t1, t0);
            }
            t_min = t0.max(t_min);
            t_max = t1.min(t_max);
            if t_max <= t_min {
                return false;
            }
        }

        true
    }
}
