use bevy_math::{Vec3, vec3};
use rand::Rng;

#[inline(always)]
pub fn reflectance(cos: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}

#[inline(always)]
pub fn refract(uv: Vec3, normal: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(normal).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * normal);
    let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs()).sqrt() * normal;
    r_out_perp + r_out_parallel
}

#[inline(always)]
pub fn reflect(v: Vec3, normal: Vec3) -> Vec3 {
    v - 2.0 * v.dot(normal) * normal
}

#[inline(always)]
pub fn near_zero(v: Vec3) -> bool {
    v.x.abs() < 1e-8 && v.y.abs() < 1e-8 && v.z.abs() < 1e-8
}

#[inline(always)]
pub fn unit_vector(v: Vec3) -> Vec3 {
    v / v.length()
}

#[inline(always)]
pub fn random_vector(min: f32, max: f32) -> Vec3 {
    let mut rng = rand::thread_rng();
    vec3(rng.gen_range(min..=max), rng.gen_range(min..=max), rng.gen_range(min..=max))
}

#[inline(always)]
pub fn random_in_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let v = vec3(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0));
        if v.length_squared() >= 1.0 {
            continue;
        }
        return v;
    }
}

#[inline(always)]
pub fn random_in_unit_disk() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let v = vec3(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0), 0.0);
        if v.length_squared() >= 1.0 {
            continue;
        }
        return v;
    }
}

#[inline(always)]
pub fn random_unit_vector() -> Vec3 {
    unit_vector(random_in_unit_sphere())
}

#[inline(always)]
pub fn random_in_hemisphere(normal: Vec3) -> Vec3 {
    let in_unit_sphere = random_in_unit_sphere();
    if in_unit_sphere.dot(normal) > 0.0 {
        in_unit_sphere
    } else {
        -in_unit_sphere
    }
}