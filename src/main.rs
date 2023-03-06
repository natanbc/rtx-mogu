mod types;
mod obj;
mod camera;
mod material;
mod util;
mod aabb;
mod bvh;
mod texture;
mod perlin;
mod aarect;

use std::cell::Cell;
use std::ptr::slice_from_raw_parts;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use bevy_math::{Vec3, vec3, Vec4, vec4, Vec4Swizzles};
use image::Rgba;
use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use crate::aarect::XZRect;
use crate::bvh::BvhNode;
use crate::camera::Camera;
use crate::material::{Dielectric, DiffuseLight, Lambertian, Metal};
use crate::obj::{HittableList, RotateX, RotateY, RotateZ, Sphere, Translate};
use crate::texture::{Checker, ImageTexture, MultiplyAdd, SolidColor, Turbulence};
use crate::types::{Color, Ray};

const RES_360P: (usize, usize) = (640, 360);
const RES_720P: (usize, usize) = (1280, 720);
const RES_1080P: (usize, usize) = (1920, 1080);
const RES_1440P: (usize, usize) = (2560, 1440);
const RES_4K: (usize, usize) = (3840, 2160);
const RES: (usize, usize) = RES_4K;
const SAMPLES_PER_PIXEL: u32 = 1500;
const MAX_DEPTH: u32 = 480;

const WIDTH: usize = RES.0;
const HEIGHT: usize = RES.1;


fn to_u32(color: Vec3, samples_per_pixel: u32) -> u32 {
    let r = color.x;
    let g = color.y;
    let b = color.z;

    let scale = 1.0 / (samples_per_pixel as f32);
    let r = (scale * r).sqrt();
    let g = (scale * g).sqrt();
    let b = (scale * b).sqrt();

    let red = (255.999 * r.clamp(0.0, 1.0)) as u8 as u32;
    let green = (255.999 * g.clamp(0.0, 1.0)) as u8 as u32;
    let blue = (255.999 * b.clamp(0.0, 1.0)) as u8 as u32;
    (0xFF << 24) | (red << 16) | (green << 8) | blue
}

fn ray_color(ray: Ray, background: Color, objs: &HittableList, depth: u32) -> Color {
    if depth == 0 {
        return Vec4::splat(0.0);
    }
    let hr = match objs.hit(ray, 0.001, f32::INFINITY) {
        Some(hr) => hr,
        None => return background,
    };

    let emitted = hr.material.emitted(hr.u, hr.v, hr.position);

    match hr.material.scatter(&ray, &hr) {
        None => emitted,
        Some((attenuation, scattered)) => {
            emitted + attenuation * ray_color(scattered, background, objs, depth - 1)
        }
    }
}

fn render_st(mut window: Window, camera: Camera, objs: HittableList) {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut rng = rand::thread_rng();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start = std::time::Instant::now();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let mut color = Vec3::splat(0.0);
                for _ in 0..SAMPLES_PER_PIXEL {
                    let du: f32 = rng.gen();
                    let dv: f32 = rng.gen();
                    let u = (x as f32 + du) / (WIDTH - 1) as f32;
                    let v = 1.0 - (y as f32 + dv) / (HEIGHT - 1) as f32;
                    let r = camera.ray(u, v);
                    let c = ray_color(r, Color::new(1.0, 1.0, 1.0, 1.0), &objs, MAX_DEPTH);
                    color += vec3(c.x, c.y, c.z) * c.w;
                }
                buffer[y * WIDTH + x] = to_u32(color, SAMPLES_PER_PIXEL);
            }
            window
                .update_with_buffer(&buffer, WIDTH, HEIGHT)
                .unwrap();
            if window.is_key_down(Key::Escape) {
                return;
            }
        }
        let elapsed = start.elapsed();
        println!("Rendered frame in {:?} ({} FPS)", elapsed, 1.0 / elapsed.as_secs_f32());
    }
}

fn render_mt(mut window: Window, camera: Camera, objs: HittableList) {
    let swap_chain = Arc::new(Mutex::new(Cell::new(vec![0; WIDTH * HEIGHT])));
    let par = std::thread::available_parallelism().unwrap().get() - 1;
    let par = par.max(1);

    let stop = Arc::new(AtomicBool::new(false));
    {
        let swap_chain = swap_chain.clone();
        let stop = stop.clone();

        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();
                let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
                struct SendPtr(*mut u32);
                unsafe impl Send for SendPtr {}
                unsafe impl Sync for SendPtr {}
                let ptr = SendPtr(buffer.as_mut_ptr());

                let pixel_index = AtomicUsize::new(0);
                let end_index = WIDTH * HEIGHT;
                let pixels_per_iter = 64;

                std::thread::scope(|s| {
                    for _ in 0..par {
                        s.spawn(|| {
                            let _ = &ptr;

                            let mut rng = rand::thread_rng();
                            loop {
                                let start_idx = pixel_index.fetch_add(pixels_per_iter, Ordering::SeqCst);
                                if start_idx >= end_index {
                                    break;
                                }

                                for idx in start_idx..(start_idx + pixels_per_iter).min(end_index) {
                                    let x = idx % WIDTH;
                                    let y = idx / WIDTH;

                                    let mut color = Vec3::splat(0.0);
                                    for _ in 0..SAMPLES_PER_PIXEL {
                                        let du: f32 = rng.gen();
                                        let dv: f32 = rng.gen();
                                        let u = (x as f32 + du) / (WIDTH - 1) as f32;
                                        let v = 1.0 - (y as f32 + dv) / (HEIGHT - 1) as f32;
                                        let r = camera.ray(u, v);
                                        let c = ray_color(r, Color::new(1.0, 1.0, 1.0, 1.0), &objs, MAX_DEPTH);
                                        color += vec3(c.x, c.y, c.z) * c.w;
                                    }
                                    let res = to_u32(color, SAMPLES_PER_PIXEL);
                                    unsafe {
                                        ptr.0.add(idx).write(res);
                                    }
                                }
                            }
                        });
                    }
                });
                let elapsed = start.elapsed();
                println!("Rendered frame in {:?} ({} FPS)", elapsed, 1.0 / elapsed.as_secs_f32());
                // let mut copy = buffer.clone();
                // for i in &mut copy {
                //     *i = u32::from_be(i.rotate_left(8));
                // }
                // image::ImageBuffer::<Rgba<u8>, _>::from_raw(WIDTH as _, HEIGHT as _, unsafe {
                //     &*slice_from_raw_parts(copy.as_ptr().cast::<u8>(), copy.len() * 4)
                // }).unwrap().save("output.png").unwrap();
                swap_chain.lock().unwrap().set(buffer);
                // break;
            }
        });
    }

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut lock = swap_chain.lock().unwrap();
        let buffer = lock.get_mut().clone();
        drop(lock);
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
    stop.store(true, Ordering::Relaxed);
}

fn main() {
    let window = Window::new(
        "RTX ON",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut objs = HittableList::new();

    let polar_to_xyz = |radius: f32, phi: f32, theta: f32| {
        vec3(
            radius * phi.sin() * theta.cos(),
            radius * phi.cos(),
            radius * phi.sin() * theta.sin(),
        )
    };

    let make_mogu = |radius: f32| {
        let mut mogu = HittableList::new();


        //rgb(164, 255, 82)
        let color = vec4(164.0/255.0, 255.0/255.0, 82.0/255.0, 1.0);

        let point = |phi: f32, theta: f32, r: f32, color: Color| {
            let x = polar_to_xyz(radius, phi, theta);
            Sphere::new(x, r, Lambertian::new(SolidColor::new(color)))
        };

        mogu.add(Sphere::new(vec3(0.0, 0.0, 0.0), radius,
             Dielectric::new(MultiplyAdd::new(
                 SolidColor::new(color),
                 SolidColor::new(Color::splat(0.5)),
                 Turbulence::new(SolidColor::new(color), 20.0)
             ), 100.0)
        ));
        let eye1 = |rotation_start: f32, rotation_end: f32, base_phi: f32, direction: f32, color: Color| {
            let rotation_start = rotation_start.to_radians();
            let rotation_end = rotation_end.to_radians();

            let n_point = 400;
            let e_radius = radius / n_point as f32 * 4.0;

            let mut spheres = HittableList::new();

            let min_radius = e_radius;
            let max_radius = e_radius * 1.5;
            for i in 0..n_point {
                let i_scale = i as f32 / n_point as f32;
                let x = rotation_start + direction * (rotation_end - rotation_start) * i_scale;

                let (a, b, c) = (0.45, -0.35, 0.2);

                let y = a * i_scale * i_scale + b * i_scale + c;
                let y = y * 0.5 + base_phi;
                let radius = min_radius + (max_radius - min_radius) * (1.0 - i_scale);
                spheres.add(point(y, x, radius, color));
            }
            BvhNode::new(&spheres.into_vec()[..])
        };
        let eye2 = |rotation_start: f32, rotation_end: f32, base_phi: f32, direction: f32, color: Color| {
            let rotation_start = rotation_start.to_radians();
            let rotation_end = rotation_end.to_radians();

            let n_point = 400;
            let e_radius = radius / 0.75 / (n_point as f32);

            let mut spheres = HittableList::new();

            let min_radius = e_radius;
            let max_radius = e_radius * 1.5;
            for i in 0..n_point {
                let i_scale = i as f32 / n_point as f32;
                let x = rotation_start + direction * (rotation_end - rotation_start) * i_scale;

                let (a, b, c) = (0.15, -0.08, 0.24);

                let y = a * i_scale * i_scale + b * i_scale + c;
                let y = y * 0.5 + base_phi;
                let radius = min_radius + (max_radius - min_radius) * i_scale;
                spheres.add(point(y, x, radius, color));
            }
            BvhNode::new(&spheres.into_vec()[..])
        };

        let black = vec4(0.0, 0.0, 0.0, 1.0);

        let base_phi = 70.0f32.to_radians();
        let base = 60.0;
        let width = 20.0;
        let gap = 40.0;
        let v_dist = 5.0f32.to_radians();
        mogu.add(eye1(base, base + width, base_phi, 1.0, black));
        mogu.add(eye1(base + width + gap, base + width + gap + width, base_phi, -1.0, black));

        mogu.add(eye2(base + width * 5.0 / 6.0, base + width, base_phi - v_dist, 1.0, black));
        mogu.add(eye2(base + width + gap / 2.0 + width / 6.0, base + width + gap / 2.0 + width / 6.0 * 2.0, base_phi - v_dist, -1.0, black));

        let mouth = |start_theta: f32, width: f32, start_phi: f32, turn_radius: f32, color: Color| {
            let mut spheres = HittableList::new();

            let start_theta = start_theta.to_radians();
            let end_theta = start_theta + width.to_radians();

            let r2 = turn_radius.powf(2.0);

            let n_point = 400;
            let radius = radius / 0.375 / (n_point as f32);

            for i in 1..(n_point + 1) {
                let i_scale = i as f32 / (n_point + 1) as f32;

                let x = start_theta + (end_theta - start_theta) * i_scale;
                let y = start_phi + (r2 - (turn_radius*(i_scale * 2.0 - 1.0)).powf(2.0)).sqrt();


                spheres.add(point(y, x, radius, color));
            }

            BvhNode::new(&spheres.into_vec()[..])
        };
        let width = 10.0;
        let center = 90.0;
        let radius = 0.1;
        let y = 90.0f32.to_radians();
        mogu.add(mouth(center - width, width, y, radius, black));
        mogu.add(mouth(center, width, y, radius, black));


        BvhNode::new(&mogu.into_vec()[..])
    };
    let mogu_center = vec3(-1.0, 0.0, -1.4);
    let mogu_radius = 1.2;

    let mogu = make_mogu(mogu_radius);
    let mogu = RotateX::new(mogu, (-60.0f32).to_radians());
    let mogu = Translate::new(mogu, mogu_center);
    objs.add(mogu);

    let mut logo = image::open("logo.png").unwrap().to_rgba8().to_owned();
    for p in logo.pixels_mut() {
        let arr = &mut p.0;
        //invert image from black to white
        let r = 255 - arr[0];
        let g = 255 - arr[1];
        let b = 255 - arr[2];
        //convert to gold
        let gold = (212.0, 175.0, 55.0);
        arr[0] = (r as f32 / 255.0 * gold.0).round() as u8;
        arr[1] = (g as f32 / 255.0 * gold.1).round() as u8;
        arr[2] = (b as f32 / 255.0 * gold.2).round() as u8;
    }
    let logo = image::imageops::rotate180(&logo);

    let logo_size = 0.5;
    let logo = XZRect::new(-logo_size/2.0, logo_size/2.0, -logo_size/2.0, logo_size/2.0, 0.0,
        Metal::new(ImageTexture::new(logo), 1.5),
    );
    let logo = RotateZ::new(logo, -35.0f32.to_radians());
    let logo = Translate::new(logo, mogu_center + polar_to_xyz(
        mogu_radius,
        35.0f32.to_radians(),
        0.0f32.to_radians())
    );
    objs.add(logo);

    objs.add(Sphere::new(vec3(20.0, 15.0, -20.0), 6.0,
        DiffuseLight::color(vec4(1.0, 1.0, 0.5, 8.0))
    ));

    let look_from = vec3(-1.0, 8.0, 3.0);
    let look_at = vec3(-1.0, 0.5, -1.0);

    let vup = vec3(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;

    let camera = Camera::new(
        look_from,
        look_at,
        vup,
        20.0,
        WIDTH as f32 / HEIGHT as f32,
        aperture,
        dist_to_focus,
    );

    render_mt(window, camera, objs);
}
