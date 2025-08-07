use crate::*;
use nannou::image::{self, GenericImage};

pub struct Samples<T> {
    pub samples: Vec<T>,
}

impl<System> Samples<System> {
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn new(initial: System, size: usize, mutation: f64) -> Self
    where
        System: ChaoticSystem + Clone,
    {
        let mut sample = Vec::with_capacity(size);

        let mut prev = initial;
        for _ in 0..size {
            sample.push(prev.clone());
            prev.mutate(mutation);
        }

        Samples { samples: sample }
    }

    pub fn update(&mut self, iterations: usize, dt: f64)
    where
        System: ChaoticSystem,
    {
        for system in &mut self.samples {
            for _ in 0..iterations {
                system.update(dt);
            }
        }
    }

    pub fn draw_line(&self, image: &mut image::DynamicImage, row: usize)
    where
        System: ChaoticSystem,
    {
        for (i, system) in self.samples.iter().enumerate() {
            image.put_pixel(i as u32, row as u32, system.color());
        }
    }
}
