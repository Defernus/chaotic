use crate::*;

pub struct Samples<T> {
    pub dimensions: Dimensions,
    pub samples: Vec<T>,
}

impl<System> Samples<System> {
    pub fn new(initial: System, dimensions: Dimensions, mutation_scales: &[f64]) -> Self
    where
        System: ChaoticSystem + Clone,
    {
        let mut samples = Vec::with_capacity(dimensions.volume());

        let mut prev = initial;
        for pos in dimensions.iter() {
            samples.push(prev.clone());
            prev.mutate(&pos, mutation_scales);
        }

        Samples {
            samples,
            dimensions,
        }
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

    pub fn iter(&self) -> impl Iterator<Item = (Vec<usize>, &System)> {
        self.samples
            .iter()
            .enumerate()
            .map(|(i, s)| (self.dimensions.index_to_pos(i), s))
    }

    // pub fn draw_2d(&self) -> image::DynamicImage
    // where
    //     System: ChaoticSystem,
    // {
    //     assert_eq!(
    //         self.dimensions.len(),
    //         2,
    //         "Expected 2D dimensions for draw_2d"
    //     );

    //     let mut image =
    //         image::DynamicImage::new_rgb8(self.dimensions[0] as u32, self.dimensions[1] as u32);

    //     for (index, pos) in self.dimensions.iter().enumerate() {
    //         let color = self.samples[index].color();
    //         image.put_pixel(pos[0] as u32, pos[1] as u32, color);
    //     }

    //     image
    // }
}
