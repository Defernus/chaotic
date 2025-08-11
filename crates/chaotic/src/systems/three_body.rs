use crate::*;
use bevy::color::{Color, Hsva, LinearRgba};
use bevy::math::DVec2;

const EPSILON: f64 = 1e-5;

#[derive(Debug, Clone, Copy)]
pub enum NBodyColorSchema {
    VelocityToRgb { v0: f64 },
    DistanceToLightness { factor: f64 },
    FirstBodyVelToGB,
}

#[derive(Debug, Clone)]
pub struct NBody {
    pub g: f64,
    pub bodies: Vec<Body>,
    pub color_schema: NBodyColorSchema,
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

impl NBody {
    pub fn new(g: f64, bodies: Vec<Body>, color_schema: NBodyColorSchema) -> Self {
        NBody {
            g,
            bodies,
            color_schema,
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Body> {
        self.bodies.iter()
    }

    /// Returns a maximum distance between bodies in the system.
    fn max_dist_sq(&self) -> f64 {
        let mut max_dist_sq = 0.0f64;
        for (i, body1) in self.iter().enumerate() {
            for (j, body2) in self.iter().enumerate() {
                if i == j {
                    continue;
                }

                let dist = (body1.position - body2.position).length_squared();
                max_dist_sq = max_dist_sq.max(dist);
            }
        }
        max_dist_sq
    }
}

impl ChaoticSystem for NBody {
    fn mutate(&mut self, pos: &[f64]) {
        for (i, &mutation) in pos.iter().enumerate() {
            let Some(body) = self.bodies.get_mut(i / 4) else {
                break;
            };

            let value = match i % 4 {
                0 => &mut body.velocity.x,
                1 => &mut body.velocity.y,
                2 => &mut body.position.x,
                3 => &mut body.position.y,
                _ => unreachable!(),
            };

            *value += mutation;
        }
    }

    fn update(&mut self, dt: f64) {
        for i in 0..self.bodies.len() {
            let body_i = &self.bodies[i];

            let mut force = DVec2::ZERO;
            for (j, body_j) in self.bodies.iter().enumerate() {
                if i == j {
                    continue;
                }

                let direction = body_j.position - body_i.position;
                let distance_sq = direction.length_squared();
                if distance_sq < EPSILON {
                    continue; // Avoid division by zero
                }
                let force_magnitude = self.g * body_j.mass * body_i.mass / distance_sq;

                force += direction.normalize() * force_magnitude;
            }

            let acceleration = force / body_i.mass;

            let body_i = &mut self.bodies[i];
            body_i.velocity += acceleration * dt;
            body_i.position += body_i.velocity * dt;
        }
    }

    fn lerp(&self, other: &Self, t: f64) -> Self {
        assert_eq!(
            self.bodies.len(),
            other.bodies.len(),
            "Mismatched body count"
        );
        let bodies = self
            .bodies
            .iter()
            .zip(&other.bodies)
            .map(|(b1, b2)| Body {
                position: b1.position.lerp(b2.position, t),
                velocity: b1.velocity.lerp(b2.velocity, t),
                mass: lerp_f64(b1.mass, b2.mass, t),
            })
            .collect::<Vec<_>>();

        NBody {
            color_schema: self.color_schema,
            g: lerp_f64(self.g, other.g, t),
            bodies,
        }
    }

    fn color(&self) -> Color {
        match self.color_schema {
            NBodyColorSchema::VelocityToRgb { v0 } => {
                if self.bodies.is_empty() {
                    return Color::BLACK;
                }

                // 1) Mean unit direction -> Hue
                let mut sum_unit = DVec2::ZERO;
                let mut sum_v_sq = 0.0;

                for body in self.iter() {
                    let len_sq = body.velocity.length_squared();
                    let u = if len_sq > 0.0 {
                        body.velocity.normalize()
                    } else {
                        DVec2::ZERO
                    };
                    sum_unit += u;
                    sum_v_sq += len_sq;
                }

                let n = self.bodies.len() as f64;
                let r = (sum_unit.length() / n).clamp(0.0, 1.0); // alignment measure in [0,1]
                let hue = if sum_unit == DVec2::ZERO {
                    0.0 // arbitrary if no direction
                } else {
                    let angle = sum_unit.y.atan2(sum_unit.x); // [-π, π]
                    ((angle / std::f64::consts::TAU) + 1.0) % 1.0 // [0,1)
                };

                // 2) Saturation from alignment (optionally sharpen with gamma)
                let gamma_s = 0.9;
                let sat = r.powf(gamma_s);

                // 3) Value from RMS speed with soft normalization
                let rms = (sum_v_sq / n).sqrt();
                let v0 = if v0 > 0.0 { v0 } else { 1.0 };
                let val = (rms / (rms + v0)).clamp(0.0, 1.0);

                let dist = 1.0 / (self.max_dist_sq() + 1.0);

                Hsva::new(hue as f32, sat as f32, val as f32, dist as f32).into()
            }

            NBodyColorSchema::DistanceToLightness { factor } => {
                let value = self.max_dist_sq() * factor + 1.0;
                let normalized_value = (1.0 / value.sqrt()) as f32;
                LinearRgba::new(normalized_value, normalized_value, normalized_value, 1.0).into()
            }

            NBodyColorSchema::FirstBodyVelToGB => {
                let Some(body) = self.bodies.get(0) else {
                    return Color::BLACK;
                };
                let velocity = body.velocity;

                let dist = 1.0 / (self.max_dist_sq() + 1.0);

                LinearRgba::new(
                    1.0 / (1.0 + velocity.x.abs() as f32),
                    1.0 / (1.0 + velocity.y.abs() as f32),
                    0.0,
                    dist as f32,
                )
                .into()
            }
        }
    }

    fn distance(&self, other: &Self) -> f64 {
        let mut total_distance = 0.0;
        for (body_a, body_b) in self.iter().zip(other.iter()) {
            let distance = body_a.velocity.distance(body_b.velocity);
            total_distance += distance;
        }

        total_distance / 3.0 // Average distance
    }
}
