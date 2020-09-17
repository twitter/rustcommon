// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! This crate is used to render a waterfall style plot of a heatmap

mod palettes;

use palettes::*;
use std::convert::TryInto;

pub use palettes::Palette;

use chrono::Utc;
use core::hash::Hash;
use core::ops::Sub;
use image::*;
use rustcommon_heatmap::*;
use rusttype::{point, FontCollection, PositionedGlyph, Scale};
use std::collections::HashMap;
use std::convert::From;
use std::time::{Duration, Instant};

pub struct WaterfallBuilder<Value> {
    output: String,
    labels: HashMap<Value, String>,
    palette: Palette,
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
            palette: Palette::Classic,
            interval: Duration::new(60, 0),
        }
    }

    pub fn label(mut self, value: Value, label: &str) -> Self {
        self.labels.insert(value, label.to_string());
        self
    }

    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
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

        let height = heatmap.windows();
        let width = heatmap.buckets();

        let mut buf = RgbImage::new(width.try_into().unwrap(), height.try_into().unwrap());

        // need to know the start time of the heatmap
        let begin_instant = heatmap.into_iter().next().unwrap().start();

        // find the bucket with the highest weight
        let mut max_weight = 0.0;
        for slice in heatmap {
            for b in slice.histogram() {
                let weight = u64::from(b.count()) as f64 / u64::from(b.width()) as f64;
                if weight > max_weight {
                    max_weight = weight;
                }
            }
        }

        let colors = match self.palette {
            Palette::Classic => CLASSIC,
            Palette::Ironbow => IRONBOW,
        };

        let mut labels = HashMap::new();
        for (k, v) in &self.labels {
            labels.insert(u64::from(*k), v);
        }

        let mut label_keys: Vec<u64> = labels.keys().cloned().collect();
        label_keys.sort();

        let mut l = 0;

        // set the pixels in the buffer
        for (y, slice) in heatmap.into_iter().enumerate() {
            for (x, b) in slice.histogram().into_iter().enumerate() {
                let weight = u64::from(b.count()) as f64 / u64::from(b.width()) as f64;
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

        let offset = now_instant - begin_instant;
        let offset = chrono::Duration::from_std(offset).unwrap();

        let begin_utc = now_datetime.checked_sub_signed(offset).unwrap();
        let mut begin = begin_instant;

        // add the timestamp labels along the left side
        for (y, slice) in heatmap.into_iter().enumerate() {
            let slice_start_utc = begin_utc
                .checked_add_signed(
                    chrono::Duration::from_std(slice.start() - begin_instant).unwrap(),
                )
                .unwrap();

            if slice.start() - begin >= self.interval {
                let label = format!("{}", slice_start_utc.to_rfc3339());
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
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap();
    let font = collection.into_font().unwrap();

    // size and scaling
    let height: f32 = size;
    let scale = Scale {
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
                    buf.put_pixel(
                        (x + x_pos).try_into().unwrap(),
                        (y + y_pos).try_into().unwrap(),
                        Rgb([255, 255, 255]),
                    );
                }
            })
        }
    }
}
