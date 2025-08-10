use crate::*;

pub struct Samples<T> {
    pub dimensions: Dimensions,
    pub samples: Vec<T>,
}

impl<System> Samples<System> {
    pub fn new(
        initial: System,
        dimensions: Dimensions,
        mutation_scales: &[f64],
        all_scale: f64,
    ) -> Self
    where
        System: ChaoticSystem + Clone,
    {
        let mut samples = Vec::with_capacity(dimensions.volume());

        let mut prev = initial;
        for pos in dimensions.iter() {
            let mutation = pos
                .into_iter()
                .zip(mutation_scales)
                .zip(dimensions.sizes())
                .map(|((cord, scale), &size)| (cord as f64 - size as f64) * scale * all_scale)
                .collect::<Vec<_>>();
            samples.push(prev.clone());
            prev.mutate(&mutation);
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
}
