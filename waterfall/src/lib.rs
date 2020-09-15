// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! This crate is used to render a waterfall style plot of a heatmap

use chrono::Utc;
use core::hash::Hash;
use core::ops::Sub;
use hsl::HSL;
use rustcommon_heatmap::*;
use rusttype::{point, FontCollection, PositionedGlyph, Scale};
use std::collections::HashMap;
use std::convert::From;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::{Duration, Instant};

pub struct WaterfallBuilder<Value> {
    output: String,
    labels: HashMap<Value, String>,
    interval: Duration,
}

impl<Value> WaterfallBuilder<Value>
where
    Value: Eq + Hash,
    u64: From<Value>,
{
    pub fn new(target: &str) -> Self {
        Self {
            output: target.to_string(),
            labels: HashMap::new(),
            interval: Duration::new(60, 0),
        }
    }

    pub fn label(mut self, value: Value, label: &str) -> Self {
        self.labels.insert(value, label.to_string());
        self
    }

    pub fn build<Count>(self, heatmap: &rustcommon_heatmap::Heatmap<Value, Count>)
    where
        Count: rustcommon_heatmap::Counter + PartialOrd + Indexing,
        Value: rustcommon_heatmap::Indexing + Sub<Output = Value> + Default + PartialOrd,
        u64: From<Count>,
        Count: From<u8>,
    {
        let now_datetime = Utc::now();
        let now_instant = Instant::now();

        let height = heatmap.slices();
        let width = heatmap.buckets();

        let mut begin_instant = Instant::now();

        let mut buffer = ImageBuffer::<ColorRgb>::new(width, height);

        let mut max_count = 0_u64;
        let mut max_width = 0_u64;
        for slice in heatmap {
            if slice.start() < begin_instant {
                begin_instant = slice.start();
            }
            for bucket in slice.histogram() {
                let c = u64::from(bucket.count());
                let w = u64::from(bucket.width());
                if c > max_count {
                    max_count = c;
                }
                if w > max_width {
                    max_width = w;
                }
            }
        }

        let mut counts = rustcommon_histogram::Histogram::<u64, u64>::new(max_count * max_width, 3);
        for slice in heatmap {
            for bucket in slice.histogram() {
                let count = u64::from(bucket.count());
                if count > 0 {
                    let multiplier = max_width / u64::from(bucket.width());
                    counts.increment(count * multiplier, 1);
                }
            }
        }

        let low = counts.percentile(1.0).unwrap();
        let mid = counts.percentile(50.0).unwrap();
        let high = counts.percentile(99.0).unwrap();
        let mut labels = HashMap::new();
        for (k, v) in &self.labels {
            labels.insert(u64::from(*k), v);
        }

        let mut label_keys: Vec<u64> = labels.keys().cloned().collect();
        label_keys.sort();

        let mut l = 0;
        for (y, slice) in heatmap.into_iter().enumerate() {
            for (x, b) in slice.histogram().into_iter().enumerate() {
                let weight = (max_width / u64::from(b.width())) * u64::from(b.count());
                let value = color_from_value(weight, low, mid, high);
                buffer.set_pixel(x, y, value);
            }
        }

        if !label_keys.is_empty() {
            let y = 0;
            let slice = heatmap.into_iter().next().unwrap();
            for (x, bucket) in slice.histogram().into_iter().enumerate() {
                let value = u64::from(bucket.value());
                if value >= label_keys[l] {
                    if let Some(label) = labels.get(&label_keys[l]) {
                        let overlay = string_buffer(label, 25.0);
                        buffer.overlay(&overlay, x, y);
                        buffer.vertical_line(
                            x,
                            ColorRgb {
                                r: 255,
                                g: 255,
                                b: 255,
                            },
                        );
                    }
                    l += 1;
                    if l >= label_keys.len() {
                        break;
                    }
                }
            }
        }

        let offset = now_instant - begin_instant;
        let offset = chrono::Duration::from_std(offset).unwrap();

        let begin_utc = now_datetime.checked_sub_signed(offset).unwrap();
        let mut begin = begin_instant;

        for (y, slice) in heatmap.into_iter().enumerate() {
            let slice_start_utc = begin_utc
                .checked_add_signed(
                    chrono::Duration::from_std(slice.start() - begin_instant).unwrap(),
                )
                .unwrap();

            if slice.start() - begin >= self.interval {
                let label = format!("{}", slice_start_utc.to_rfc3339());
                let overlay = string_buffer(&label, 25.0);
                buffer.overlay(&overlay, 0, y + 2);
                buffer.horizontal_line(
                    y,
                    ColorRgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                );
                begin += self.interval;
            }
        }

        let _ = buffer.write_png(&self.output);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

struct ImageBuffer<T> {
    buffer: Vec<Vec<T>>,
    height: usize,
    width: usize,
}

fn string_buffer(string: &str, size: f32) -> ImageBuffer<ColorRgb> {
    // load font
    let font_data = dejavu::sans_mono::regular();
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap();
    let font = collection.into_font().unwrap();

    // size and scaling
    let height: f32 = size;
    let pixel_height = height.ceil() as usize;
    let scale = Scale {
        x: height * 1.0,
        y: height,
    };

    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let glyphs: Vec<PositionedGlyph> = font.layout(string, scale, offset).collect();

    let width = glyphs
        .iter()
        .map(|g| g.unpositioned().h_metrics().advance_width)
        .fold(0.0, |x, y| x + y)
        .ceil() as usize;

    let mut overlay = ImageBuffer::<ColorRgb>::new(width, pixel_height);

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = (x as i32 + bb.min.x) as usize;
                let y = (y as i32 + bb.min.y) as usize;
                if v > 0.25 {
                    overlay.set_pixel(
                        x,
                        y,
                        ColorRgb {
                            r: 255,
                            g: 255,
                            b: 255,
                        },
                    );
                }
            })
        }
    }

    overlay
}

/// maps a value to a color based on a low point, mid point, and high point
/// values below low will clip to black
/// mid point is the transition between luminosity (black-blue) and hue (blue->red) ramps
/// values above high will clip to red
fn color_from_value(value: u64, low: u64, mid: u64, high: u64) -> ColorRgb {
    let hsl = if value < low {
        HSL {
            h: 250.0,
            s: 1.0,
            l: 0.0,
        }
    } else if value < mid {
        HSL {
            h: 250.0,
            s: 1.0,
            l: (value as f64 / mid as f64) * 0.5,
        }
    } else if value < high {
        HSL {
            h: 250.0 - (250.0 * (value - mid) as f64 / high as f64),
            s: 1.0,
            l: 0.5,
        }
    } else {
        HSL {
            h: 0.0,
            s: 1.0,
            l: 0.5,
        }
    };

    let (r, g, b) = hsl.to_rgb();

    ColorRgb { r, g, b }
}

impl ImageBuffer<ColorRgb> {
    pub fn new(width: usize, height: usize) -> ImageBuffer<ColorRgb> {
        let background = ColorRgb { r: 0, g: 0, b: 0 };
        let mut row = Vec::<ColorRgb>::with_capacity(width);
        for _ in 0..width {
            row.push(background);
        }
        let mut buffer = Vec::<Vec<ColorRgb>>::with_capacity(height);
        for _ in 0..height {
            buffer.push(row.clone());
        }
        ImageBuffer {
            buffer,
            height,
            width,
        }
    }

    pub fn write_png(self, file: &str) -> Result<(), &'static str> {
        let mut buffer = Vec::<u8>::with_capacity(self.height * self.width);
        for row in 0..self.height {
            for col in 0..self.width {
                let pixel = self.buffer[row][col];
                buffer.push(pixel.r);
                buffer.push(pixel.g);
                buffer.push(pixel.b);
            }
        }
        let path = &Path::new(&file);
        if let Ok(file) = File::create(path) {
            let w = BufWriter::new(file);
            let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
            encoder.set_color(png::ColorType::RGB);
            encoder.set_depth(png::BitDepth::Eight);
            if let Ok(mut writer) = encoder.write_header() {
                if writer.write_image_data(&buffer).is_ok() {
                    Ok(())
                } else {
                    Err("Error writing PNG data")
                }
            } else {
                Err("Error writing PNG header")
            }
        } else {
            Err("Error creating file")
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: ColorRgb) {
        if x < self.width && y < self.height {
            self.buffer[y][x] = value;
        }
    }

    pub fn overlay(&mut self, other: &ImageBuffer<ColorRgb>, x: usize, y: usize) {
        let ignore = ColorRgb { r: 0, g: 0, b: 0 };
        for sx in 0..other.width {
            for sy in 0..other.height {
                if (other.buffer[sy][sx] != ignore)
                    && (((sy + y) < self.height) && ((sx + x) < self.width))
                {
                    self.buffer[(sy + y)][(sx + x)] = other.buffer[sy][sx];
                }
            }
        }
    }

    pub fn horizontal_line(&mut self, y: usize, color: ColorRgb) {
        for x in 0..self.width {
            self.buffer[y][x] = color;
        }
    }

    pub fn vertical_line(&mut self, x: usize, color: ColorRgb) {
        for y in 0..self.height {
            self.buffer[y][x] = color;
        }
    }
}
