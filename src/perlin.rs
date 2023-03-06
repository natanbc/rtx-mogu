use bevy_math::Vec3;
use crate::util::{random_vector, unit_vector};
use rand::seq::SliceRandom;
use crate::types::Point3;

const POINT_COUNT: usize = 256;

fn generate_perm() -> Vec<usize> {
    let mut res = Vec::with_capacity(POINT_COUNT);
    for i in 0..POINT_COUNT {
        res.push(i);
    }
    let mut rng = rand::thread_rng();
    res.shuffle(&mut rng);

    res
}

#[derive(Clone)]
pub struct Perlin {
    vecs: Vec<Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    pub fn new() -> Self {
        let mut vecs = Vec::new();
        for _ in 0..POINT_COUNT {
            vecs.push(unit_vector(random_vector(-1.0, 1.0)))
        }
        let perm_x = generate_perm();
        let perm_y = generate_perm();
        let perm_z = generate_perm();
        Self {
            vecs,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, p: Point3) -> f32 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as isize;
        let j = p.y.floor() as isize;
        let k = p.z.floor() as isize;

        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut acc = 0.0;

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let c = self.vecs[
                        self.perm_x[((i + di) & 255) as usize] ^
                        self.perm_y[((j + dj) & 255) as usize] ^
                        self.perm_z[((k + dk) & 255) as usize]
                    ];
                    let i_f = di as f32;
                    let j_f = dj as f32;
                    let k_f = dk as f32;
                    let weight = Vec3::new(u - i_f, v - j_f, w - k_f);
                    acc += (i_f * uu + (1.0 - i_f) * (1.0 - uu)) *
                           (j_f * vv + (1.0 - j_f) * (1.0 - vv)) *
                           (k_f * ww + (1.0 - k_f) * (1.0 - ww)) *
                           c.dot(weight);
                }
            }
        }

        acc
    }

    pub fn normalized_noise(&self, p: Point3) -> f32 {
        self.noise(p) * 0.5 + 0.5
    }

    pub fn turbulence(&self, p: Point3, depth: u32) -> f32 {
        let mut accum = 0.0;
        let mut p = p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(p);
            weight *= 0.5;
            p *= 2.0;
        }

        accum.abs()
    }
}
