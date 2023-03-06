use std::cmp::Ordering;
use std::sync::Arc;
use rand::Rng;
use crate::aabb::AABB;
use crate::obj::{HitResult, Hittable};
use crate::types::Ray;

pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: AABB,
}

unsafe impl Send for BvhNode {}
unsafe impl Sync for BvhNode {}

impl BvhNode {
    pub fn new(objects: &[Arc<dyn Hittable + Send>]) -> Self {
        debug_assert_ne!(objects.len(), 0, "List cannot be empty");

        let axis = rand::thread_rng().gen_range(0..=2);
        let cmp = |a: &Arc<dyn Hittable + Send>, b: &Arc<dyn Hittable + Send>| {
            let a_min = a.bounding_box().min.to_array()[axis];
            let b_min = b.bounding_box().min.to_array()[axis];
            a_min.total_cmp(&b_min)
        };

        let (left, right) = match objects.len() {
            0 => panic!("No objects"),
            1 => (objects[0].clone(), objects[0].clone()),
            2 => {
                let a = objects[0].clone();
                let b = objects[1].clone();
                if cmp(&a, &b) == Ordering::Greater {
                    (b, a)
                } else {
                    (a, b)
                }
            },
            _ => {
                let mut copy = objects.to_vec();
                copy.sort_by(cmp);

                let mid = copy.len() / 2;
                (
                    Arc::new(Self::new(&copy[..mid])) as _,
                    Arc::new(Self::new(&copy[mid..])) as _,
                )
            }
        };
        let bbox = AABB::surrounding_box(left.bounding_box(), right.bounding_box());
        Self {
            left,
            right,
            bbox,
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        if !self.bbox.hit(ray, t_min, t_max) {
            return None;
        }

        let left = self.left.hit(ray, t_min, t_max);
        if let Some(res) = left.as_ref() {
            let right = self.right.hit(ray, t_min, res.t);
            if right.is_some() {
                right
            } else {
                left
            }
        } else {
            self.right.hit(ray, t_min, t_max)
        }
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
