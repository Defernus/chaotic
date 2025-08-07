use crate::*;
use nannou::{color, image};

#[derive(Debug, Clone)]
pub struct DoublePendulum {
    pub length1: f64,
    pub length2: f64,
    pub mass1: f64,
    pub mass2: f64,
    pub angle1: f64,
    pub angle2: f64,
    pub angular_velocity1: f64,
    pub angular_velocity2: f64,
    pub dampening: f64,
}

impl DoublePendulum {
    pub fn new(length1: f64, length2: f64, mass1: f64, mass2: f64) -> Self {
        DoublePendulum {
            length1,
            length2,
            mass1,
            mass2,
            angle1: 0.0,
            angle2: 0.0,
            angular_velocity1: 0.0,
            angular_velocity2: 0.0,
            dampening: 0.000001,
        }
    }

    pub fn with_angle1(mut self, angle1: f64) -> Self {
        self.angle1 = angle1;
        self
    }

    pub fn with_angle2(mut self, angle2: f64) -> Self {
        self.angle2 = angle2;
        self
    }

    pub fn update(&mut self, gravity: f64) {
        let num = -gravity * (2.0 * self.mass1 + self.mass2) * self.angle1.sin()
            - self.mass2 * gravity * (self.angle1 - 2.0 * self.angle2).sin()
            - 2.0
                * (self.angle1 - self.angle2).sin()
                * self.mass2
                * (self.angular_velocity2 * self.angular_velocity2 * self.length2
                    + self.angular_velocity1
                        * self.angular_velocity1
                        * self.length1
                        * (self.angle1 - self.angle2).cos());
        let den = self.length1
            * (2.0 * self.mass1 + self.mass2
                - self.mass2 * (2.0 * self.angle1 - 2.0 * self.angle2).cos());
        let accel1 = num / den;

        let num = 2.0
            * (self.angle1 - self.angle2).sin()
            * (self.angular_velocity1
                * self.angular_velocity1
                * self.length1
                * (self.mass1 + self.mass2)
                + gravity * (self.mass1 + self.mass2) * self.angle1.cos()
                + self.angular_velocity2
                    * self.angular_velocity2
                    * self.length2
                    * self.mass2
                    * (self.angle1 - self.angle2).cos());
        let den = self.length2
            * (2.0 * self.mass1 + self.mass2
                - self.mass2 * (2.0 * self.angle1 - 2.0 * self.angle2).cos());
        let accel2 = num / den;

        self.angular_velocity1 += accel1;
        self.angular_velocity2 += accel2;
        self.angle1 += self.angular_velocity1;
        self.angle2 += self.angular_velocity2;
        self.angular_velocity1 *= 1.0 - self.dampening;
        self.angular_velocity2 *= 1.0 - self.dampening;
    }

    pub fn color(&self) -> image::Rgb<u8> {
        let rgb: color::Rgb = color::Hsv::new(
            (normalize_angle(self.angle1) * 360.0) as f32,
            ((self.angle2.sin() + 1.0) * 0.5) as f32,
            1.0,
        )
        .into();

        image::Rgb([
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        ])
    }
}
