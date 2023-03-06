use bevy_math::vec4;
use image::RgbaImage;
use crate::perlin::Perlin;
use crate::types::{Color, Point3};

pub trait Texture {
    //Hack to implement transparency for images
    fn hack_solid(&self, _: f32, _: f32, _: Point3) -> bool {
        true
    }

    fn value(&self, u: f32, v: f32, point: Point3) -> Color;
}

#[derive(Copy, Clone)]
pub struct SolidColor {
    color: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        Self {
            color,
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, _: f32, _: f32, _: Point3) -> Color {
        self.color
    }
}

#[derive(Copy, Clone)]
pub struct Checker<E: Texture, O: Texture> {
    even: E,
    odd: O,
    mul: f32,
}

impl<E: Texture, O: Texture> Checker<E, O> {
    pub fn new(even: E, odd: O) -> Self {
        Self {
            even,
            odd,
            mul: 10.0,
        }
    }
}

impl Checker<SolidColor, SolidColor> {
    pub fn color(even: Color, odd: Color) -> Self {
        Self::new(SolidColor::new(even), SolidColor::new(odd))
    }
}

impl<E: Texture, O: Texture> Texture for Checker<E, O> {
    fn value(&self, u: f32, v: f32, point: Point3) -> Color {
        let sines = (self.mul * point.x).sin() * (self.mul * point.y).sin() * (self.mul * point.z).sin();
        if sines < 0.0 {
            self.odd.value(u, v, point)
        } else {
            self.even.value(u, v, point)
        }
    }
}

#[derive(Clone)]
pub struct Noise<T> {
    texture: T,
    scale: f32,
    noise: Perlin,
}

impl<T> Noise<T> {
    pub fn new(texture: T, scale: f32) -> Self {
        Self {
            texture,
            scale,
            noise: Perlin::new(),
        }
    }
}

impl<T: Texture> Texture for Noise<T> {
    fn value(&self, u: f32, v: f32, point: Point3) -> Color {
        self.texture.value(u, v, point) * self.noise.normalized_noise(point * self.scale)
    }
}

#[derive(Clone)]
pub struct Turbulence<T> {
    texture: T,
    scale: f32,
    noise: Perlin,
}

impl<T> Turbulence<T> {
    pub fn new(texture: T, scale: f32) -> Self {
        Self {
            texture,
            scale,
            noise: Perlin::new(),
        }
    }
}

impl<T: Texture> Texture for Turbulence<T> {
    fn value(&self, u: f32, v: f32, point: Point3) -> Color {
        self.texture.value(u, v, point) * self.noise.turbulence(point * self.scale, 7)
    }
}

#[derive(Clone)]
pub struct TurbulencePhase<T> {
    texture: T,
    scale: f32,
    noise: Perlin,
}

impl<T> TurbulencePhase<T> {
    pub fn new(texture: T, scale: f32) -> Self {
        Self {
            texture,
            scale,
            noise: Perlin::new(),
        }
    }
}

impl<T: Texture> Texture for TurbulencePhase<T> {
    fn value(&self, u: f32, v: f32, point: Point3) -> Color {
        let mul = (self.scale * point.z + 10.0 * self.noise.turbulence(point, 7)).sin() * 0.5 + 0.5;
        self.texture.value(u, v, point) * mul
    }
}

#[derive(Copy, Clone)]
pub struct MultiplyAdd<A: Texture, B: Texture, C: Texture> {
    a: A,
    b: B,
    c: C,
}

impl<A: Texture, B: Texture, C: Texture> MultiplyAdd<A, B, C> {
    pub fn new(a: A, b: B, c: C) -> Self {
        Self {
            a,
            b,
            c,
        }
    }
}

impl<A: Texture, B: Texture, C: Texture> Texture for MultiplyAdd<A, B, C> {
    fn value(&self, u: f32, v: f32, point: Point3) -> Color {
        let a = self.a.value(u, v, point);
        let b = self.b.value(u, v, point);
        let c = self.c.value(u, v, point);
        (a + b * c).clamp(Color::splat(0.0), Color::splat(1.0))
    }
}

#[derive(Clone)]
pub struct ImageTexture {
    image: RgbaImage,
}

impl ImageTexture {
    pub fn new(image: RgbaImage) -> Self {
        Self {
            image,
        }
    }
}

impl Texture for ImageTexture {
    fn hack_solid(&self, u: f32, v: f32, _: Point3) -> bool {
        let u = 1.0 - u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let mut i = (self.image.width() as f32 * u) as u32;
        let mut j = (self.image.height() as f32 * v) as u32;
        if i >= self.image.width() {
            i = self.image.width() - 1;
        }
        if j >= self.image.height() {
            j = self.image.height() - 1;
        }

        let pixel = self.image.get_pixel(i, j).0;
        pixel[3] > 10
    }

    fn value(&self, u: f32, v: f32, _point: Point3) -> Color {
        let u = 1.0 - u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let mut i = (self.image.width() as f32 * u) as u32;
        let mut j = (self.image.height() as f32 * v) as u32;
        if i >= self.image.width() {
            i = self.image.width() - 1;
        }
        if j >= self.image.height() {
            j = self.image.height() - 1;
        }

        let scale = 1.0 / 255.0;
        let pixel = self.image.get_pixel(i, j).0;
        let (r, g, b, a) = (pixel[0] as f32 * scale, pixel[1] as f32 * scale, pixel[2] as f32 * scale, pixel[3] as f32 * scale);
        vec4(r, g, b, a)
    }
}
