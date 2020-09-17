// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rand::thread_rng;
use rand_distr::*;
use rustcommon_logger::*;
use rustcommon_waterfall::Palette;
use rustcommon_waterfall::WaterfallBuilder;
use std::time::Duration;
use std::time::Instant;

fn main() {
    Logger::new()
        .label("simulator")
        .level(Level::Debug)
        .init()
        .expect("Failed to initialize logger");

    info!("Welcome to the simulator!");

    for shape in &[
        Shape::Cauchy,
        Shape::Normal,
        Shape::Uniform,
        Shape::Triangular,
        Shape::Gamma,
    ] {
        simulate(*shape);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Shape {
    Cauchy,
    Normal,
    Uniform,
    Triangular,
    Gamma,
}

pub fn simulate(shape: Shape) {
    println!("simulating for {:?}", shape);
    let duration = 120;

    let mut heatmap =
        rustcommon_heatmap::Heatmap::<u64, u64>::new(1_000_000, 3, 120, Duration::new(1, 0));

    let cauchy = Cauchy::new(500_000.0, 2_000.00).unwrap();
    let normal = Normal::new(200_000.0, 100_000.0).unwrap();
    let uniform = Uniform::new_inclusive(10_000.0, 200_000.0);
    let triangular = Triangular::new(1.0, 200_000.0, 50_000.0).unwrap();
    let gamma = Gamma::new(2.0, 2.0).unwrap();

    let start = std::time::Instant::now();
    let mut rng = thread_rng();

    loop {
        let now = std::time::Instant::now();
        if now - start >= std::time::Duration::new(duration, 0) {
            break;
        }
        let value: f64 = match shape {
            Shape::Cauchy => cauchy.sample(&mut rng),
            Shape::Normal => normal.sample(&mut rng),
            Shape::Uniform => uniform.sample(&mut rng),
            Shape::Triangular => triangular.sample(&mut rng),
            Shape::Gamma => gamma.sample(&mut rng) * 1_000_000.0,
        };
        let value = value.floor() as u64;
        if value != 0 {
            heatmap.increment(Instant::now(), value, 1);
        }
    }

    let shape_name = match shape {
        Shape::Cauchy => "cauchy",
        Shape::Normal => "normal",
        Shape::Uniform => "uniform",
        Shape::Triangular => "triangular",
        Shape::Gamma => "gamma",
    };

    for palette in [Palette::Classic, Palette::Ironbow].iter() {
        let palette_name = match palette {
            Palette::Classic => "classic",
            Palette::Ironbow => "ironbow",
        };

        let filename = format!("{}_{}.png", shape_name, palette_name);

        WaterfallBuilder::new(&filename)
            .label(100, "100")
            .label(1000, "1000")
            .label(10000, "10000")
            .label(100000, "100000")
            .palette(*palette)
            .build(&heatmap);
    }
}
