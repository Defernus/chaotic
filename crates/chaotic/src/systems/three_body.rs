use crate::*;
use nannou::glam::{DVec2, DVec3};
use nannou::image;

const EPSILON: f64 = 1e-5;

#[derive(Debug, Clone)]
pub struct ThreeBody {
    pub g: f64,
    pub a: Body,
    pub b: Body,
    pub c: Body,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub position: DVec2,
    pub velocity: DVec2,
    pub mass: f64,
}

impl Body {
    pub fn new(mass: f64, position: DVec2, velocity: DVec2) -> Self {
        Body {
            position,
            velocity,
            mass,
        }
    }
}

impl ThreeBody {
    pub fn new(g: f64, a: Body, b: Body, c: Body) -> Self {
        ThreeBody { g, a, b, c }
    }

    pub fn raw_rgb(&self) -> DVec3 {
        let r = val_to_channel(self.a.velocity);
        let g = val_to_channel(self.b.velocity);
        let b = val_to_channel(self.c.velocity);

        DVec3::new(r, g, b)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Body> {
        [&self.a, &self.b, &self.c].into_iter()
    }
}

impl ChaoticSystem for ThreeBody {
    fn mutate(&mut self, mutation: f64) {
        self.a.velocity.x += mutation * (rand::random::<f64>() - 0.5);
        self.a.velocity.y += mutation * (rand::random::<f64>() - 0.5);
        self.b.velocity.x += mutation * (rand::random::<f64>() - 0.5);
        self.b.velocity.y += mutation * (rand::random::<f64>() - 0.5);
        self.c.velocity.x += mutation * (rand::random::<f64>() - 0.5);
        self.c.velocity.y += mutation * (rand::random::<f64>() - 0.5);
    }

    fn update(&mut self, dt: f64) {
        let mut bodies = [&mut self.a, &mut self.b, &mut self.c];
        for i in 0..bodies.len() {
            let body_i = &bodies[i];
            let mut force = DVec2::ZERO;

            for j in 0..bodies.len() {
                if i == j {
                    continue;
                }

                let body_j = &bodies[j];
                let direction = body_j.position - body_i.position;
                let distance = direction.length();
                if distance < EPSILON {
                    continue; // Avoid division by zero
                }
                let force_magnitude = self.g * body_i.mass * body_j.mass / (distance * distance);

                force += direction.normalize() * force_magnitude;
            }

            let acceleration = force / body_i.mass;
            let body_i = &mut bodies[i];
            body_i.velocity += acceleration * dt;
            body_i.position += body_i.velocity * dt;
        }
    }

    fn lerp(&self, other: &Self, t: f64) -> Self {
        let a = Body {
            position: self.a.position.lerp(other.a.position, t),
            velocity: self.a.velocity.lerp(other.a.velocity, t),
            mass: lerp_f64(self.a.mass, other.a.mass, t),
        };
        let b = Body {
            position: self.b.position.lerp(other.b.position, t),
            velocity: self.b.velocity.lerp(other.b.velocity, t),
            mass: lerp_f64(self.b.mass, other.b.mass, t),
        };
        let c = Body {
            position: self.c.position.lerp(other.c.position, t),
            velocity: self.c.velocity.lerp(other.c.velocity, t),
            mass: lerp_f64(self.c.mass, other.c.mass, t),
        };

        ThreeBody {
            g: lerp_f64(self.g, other.g, t),
            a,
            b,
            c,
        }
    }

    fn color(&self) -> image::Rgba<u8> {
        let rgb = self.raw_rgb();
        image::Rgba([
            (rgb.x * 255.0) as u8,
            (rgb.y * 255.0) as u8,
            (rgb.z * 255.0) as u8,
            255, // Alpha channel
        ])
    }
}

fn val_to_channel(vel: DVec2) -> f64 {
    let vel = vel.normalize_or_zero();

    // let angle = if vel == DVec2::ZERO {
    //     0.0
    // } else {
    //     DVec2::X.angle_between(vel)
    // };

    // normalize_angle(angle)
    (vel.x + vel.y + 2.0) / 4.0
}
