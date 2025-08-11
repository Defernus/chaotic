use crate::*;
use bevy::color::{Color, LinearRgba};
use bevy::math::DVec2;

#[derive(Debug, Clone, Copy)]
pub enum MandelbrotColorSchema {
    Distance,
}

#[derive(Debug, Clone)]
pub struct Mandelbrot {
    pub color_schema: MandelbrotColorSchema,
    pub z: DVec2,
    pub c: DVec2,
}

impl Mandelbrot {
    pub fn new(color_schema: MandelbrotColorSchema) -> Self {
        // z = z*z + c
        Mandelbrot {
            color_schema,
            z: DVec2::ZERO,
            c: DVec2::ZERO,
        }
    }
}

impl ChaoticSystem for Mandelbrot {
    fn mutate(&mut self, pos: &[f64]) {
        self.c = DVec2::new(
            pos.get(0).copied().unwrap_or_default(),
            pos.get(1).copied().unwrap_or_default(),
        );
    }

    fn update(&mut self, _dt: f64) {
        self.z = DVec2::new(
            self.z.x * self.z.x - self.z.y * self.z.y,
            2.0 * self.z.x * self.z.y,
        ) + self.c;
    }

    fn lerp(&self, other: &Self, t: f64) -> Self {
        Mandelbrot {
            color_schema: self.color_schema,
            z: self.z.lerp(other.z, t),
            c: self.c.lerp(other.c, t),
        }
    }

    fn color(&self) -> Color {
        match self.color_schema {
            MandelbrotColorSchema::Distance => {
                let distance = 1.0 / (1.0 + self.z.length_squared() as f32);
                LinearRgba::new(1.0, 1.0, 1.0, distance).into()
            }
        }
    }

    fn distance(&self, other: &Self) -> f64 {
        (self.z - other.z).length_squared()
    }
}
