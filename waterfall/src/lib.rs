// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! This crate is used to render a waterfall style plot of a heatmap

mod palettes;

pub use palettes::Palette;

use chrono::Utc;
use image::*;
use palettes::*;
use rustcommon_heatmap::*;
use rusttype::{point, Font, PositionedGlyph, Scale as TypeScale};

use core::hash::Hash;
use core::ops::Sub;
use std::collections::HashMap;
use std::convert::{From, TryInto};

#[derive(Copy, Clone)]
/// Used to configure various strategies for mapping values to colors
pub enum Scale {
    /// Use a linear mapping
    Linear,
    /// Use a logarithmic mapping
    Logarithmic,
}

pub struct WaterfallBuilder<Value> {
    output: String,
    labels: HashMap<Value, String>,
    palette: Palette,
    interval: Duration,
    scale: Scale,
    smooth: Option<f32>,
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
            palette: Palette::Classic,
            interval: Duration::new(60, 0),
            scale: Scale::Linear,
            smooth: None,
        }
    }

    /// Adds a label to the horizontal axis at the specified value
    pub fn label(mut self, value: Value, label: &str) -> Self {
        self.labels.insert(value, label.to_string());
        self
    }

    /// Sets the color palette for the waterfall
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Select a color scale for the waterfall
    pub fn scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    /// Set a smoothing on the waterfall which is applied before colorization
    pub fn smooth(mut self, sigma: Option<f32>) -> Self {
        self.smooth = sigma;
        self
    }

    // get the scaled weight for a bucket count / width
    fn weight(&self, count: u64, width: u64) -> f64 {
        match self.scale {
            Scale::Linear => count as f64 / width as f64,
            Scale::Logarithmic => (count as f64 / width as f64).log2(),
        }
    }

    // find the bucket with the highest weight
    fn max_weight<Count>(&self, heatmap: &rustcommon_heatmap::Heatmap<Value, Count>) -> f64
    where
        Count: rustcommon_heatmap::Counter + PartialOrd + Indexing,
        Value: rustcommon_heatmap::Indexing + Sub<Output = Value> + Default + PartialOrd,
        u64: From<Count>,
        Count: From<u8>,
    {
        let mut max_weight = 0.0;
        for slice in heatmap {
            for b in slice.histogram() {
                let weight = self.weight(u64::from(b.count()), u64::from(b.width()));
                if weight > max_weight {
                    max_weight = weight;
                }
            }
        }
        max_weight
    }

    /// Generate the waterfall from the provided heatmap
    pub fn build<Count>(self, heatmap: &rustcommon_heatmap::Heatmap<Value, Count>)
    where
        Count: rustcommon_heatmap::Counter + PartialOrd + Indexing,
        Value: rustcommon_heatmap::Indexing + Sub<Output = Value> + Default + PartialOrd,
        u64: From<Count>,
        Count: From<u8>,
    {
        let now_datetime = Utc::now();
        let now_instant = Instant::now();

        let height = heatmap.windows();
        let width = heatmap.buckets();

        let mut buf = RgbImage::new(width.try_into().unwrap(), height.try_into().unwrap());

        // need to know the start time of the heatmap
        let begin_instant = heatmap.into_iter().next().unwrap().start();

        let max_weight = self.max_weight(heatmap);

        let colors = match self.palette {
            Palette::Classic => CLASSIC,
            Palette::Ironbow => IRONBOW,
        };

        let mut labels = HashMap::new();
        for (k, v) in &self.labels {
            labels.insert(u64::from(*k), v);
        }

        let mut label_keys: Vec<u64> = labels.keys().cloned().collect();
        label_keys.sort_unstable();

        let mut l = 0;

        if let Some(sigma) = self.smooth {
            // NOTE: this won't work properly if the palette is > 256 colors

            // build grayscale buffer
            for (y, slice) in heatmap.into_iter().enumerate() {
                for (x, b) in slice.histogram().into_iter().enumerate() {
                    let weight = self.weight(u64::from(b.count()), u64::from(b.width()));
                    let scaled_weight = weight / max_weight;
                    let index = (scaled_weight * (colors.len() - 1) as f64).round() as u8;
                    buf.put_pixel(
                        x.try_into().unwrap(),
                        y.try_into().unwrap(),
                        Rgb([index, index, index]),
                    );
                }
            }

            // apply a blur to smooth
            buf = image::imageops::blur(&buf, sigma);

            // colorize the buffer
            for x in 0..buf.width() {
                for y in 0..buf.height() {
                    let index = buf.get_pixel(x, y).0[0];
                    let color = colors[index as usize];
                    buf.put_pixel(x, y, Rgb([color.r, color.g, color.b]));
                }
            }
        } else {
            // set the pixels in the buffer
            for (y, slice) in heatmap.into_iter().enumerate() {
                for (x, b) in slice.histogram().into_iter().enumerate() {
                    let weight = self.weight(u64::from(b.count()), u64::from(b.width()));
                    let scaled_weight = weight / max_weight;
                    let index = (scaled_weight * (colors.len() - 1) as f64).round() as usize;
                    let color = colors[index];
                    buf.put_pixel(
                        x.try_into().unwrap(),
                        y.try_into().unwrap(),
                        Rgb([color.r, color.g, color.b]),
                    );
                }
            }
        }

        // add the horizontal labels across the top
        if !label_keys.is_empty() {
            let slice = heatmap.into_iter().next().unwrap();
            for (x, bucket) in slice.histogram().into_iter().enumerate() {
                let value = u64::from(bucket.value());
                if value >= label_keys[l] {
                    if let Some(label) = labels.get(&label_keys[l]) {
                        render_text(label, 25.0, x, 0, &mut buf);
                        for y in 0..height {
                            buf.put_pixel(
                                x.try_into().unwrap(),
                                y.try_into().unwrap(),
                                Rgb([255, 255, 255]),
                            );
                        }
                    }
                    l += 1;
                    if l >= label_keys.len() {
                        break;
                    }
                }
            }
        }

        let offset = std::time::Duration::from_nanos((now_instant - begin_instant).as_nanos() as _);
        let offset = chrono::Duration::from_std(offset).unwrap();

        let begin_utc = now_datetime.checked_sub_signed(offset).unwrap();
        let mut begin = begin_instant;

        // add the timestamp labels along the left side
        for (y, slice) in heatmap.into_iter().enumerate() {
            let duration =
                std::time::Duration::from_nanos((slice.start() - begin_instant).as_nanos() as _);
            let slice_start_utc = begin_utc
                .checked_add_signed(chrono::Duration::from_std(duration).unwrap())
                .unwrap();

            if slice.start() - begin >= self.interval {
                let label = slice_start_utc.to_rfc3339().to_string();
                render_text(&label, 25.0, 0, y + 2, &mut buf);
                for x in 0..width {
                    buf.put_pixel(
                        x.try_into().unwrap(),
                        y.try_into().unwrap(),
                        Rgb([255, 255, 255]),
                    );
                }
                begin += self.interval;
            }
        }
        buf.save(&self.output).unwrap();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

fn render_text(string: &str, size: f32, x_pos: usize, y_pos: usize, buf: &mut RgbImage) {
    // load font
    let font_data = dejavu::sans_mono::regular();
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

    // size and scaling
    let height: f32 = size;
    let scale = TypeScale {
        x: height * 1.0,
        y: height,
    };

    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let glyphs: Vec<PositionedGlyph> = font.layout(string, scale, offset).collect();

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = (x as i32 + bb.min.x) as usize;
                let y = (y as i32 + bb.min.y) as usize;
                if v > 0.25 {
                    let x = (x + x_pos).try_into().unwrap();
                    let y = (y + y_pos).try_into().unwrap();
                    if x < buf.width() && y < buf.height() {
                        buf.put_pixel(x, y, Rgb([255, 255, 255]));
                    }
                }
            })
        }
    }
}
