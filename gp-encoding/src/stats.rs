// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

#[derive(Debug, Clone)]
pub struct BandStats {
    pub band_index: u32,
    pub dtype_name: String,
    pub total_cells: u64,
    pub valued_cells: u64,
    pub fill_cells: u64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
    pub sum_sq: f64,
}

impl BandStats {
    pub fn mean(&self) -> f64 {
        if self.valued_cells == 0 {
            0.0
        } else {
            self.sum / self.valued_cells as f64
        }
    }

    pub fn stddev(&self) -> f64 {
        if self.valued_cells <= 1 {
            0.0
        } else {
            let n = self.valued_cells as f64;
            let variance = (self.sum_sq - (self.sum * self.sum) / n) / (n - 1.0);
            variance.max(0.0).sqrt()
        }
    }

    pub fn fill_percentage(&self) -> f64 {
        if self.total_cells == 0 {
            0.0
        } else {
            (self.fill_cells as f64 / self.total_cells as f64) * 100.0
        }
    }

    pub fn valued_percentage(&self) -> f64 {
        if self.total_cells == 0 {
            0.0
        } else {
            (self.valued_cells as f64 / self.total_cells as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionReport {
    pub num_chunks: u64,
    pub chunk_size: u64,
    pub chunk_level: u32,
    pub refinement_level: u32,
    pub bands: Vec<BandStats>,
}

#[derive(Debug, Clone)]
pub struct SourceRasterReport {
    pub width: usize,
    pub height: usize,
    pub total_pixels: u64,
    pub bands: Vec<BandStats>,
}

pub struct BandStatsCollector {
    band_index: u32,
    dtype_name: String,
    total_cells: u64,
    valued_cells: u64,
    min: f64,
    max: f64,
    sum: f64,
    sum_sq: f64,
}

impl BandStatsCollector {
    pub fn new(band_index: u32, dtype_name: String) -> Self {
        Self {
            band_index,
            dtype_name,
            total_cells: 0,
            valued_cells: 0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    pub fn record_value(&mut self, value: f64) {
        self.valued_cells += 1;
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sum += value;
        self.sum_sq += value * value;
    }

    pub fn set_total_cells(&mut self, total: u64) {
        self.total_cells = total;
    }

    pub fn merge(&mut self, other: &BandStatsCollector) {
        self.valued_cells += other.valued_cells;
        if other.min < self.min {
            self.min = other.min;
        }
        if other.max > self.max {
            self.max = other.max;
        }
        self.sum += other.sum;
        self.sum_sq += other.sum_sq;
    }

    pub fn finish(self) -> BandStats {
        BandStats {
            band_index: self.band_index,
            dtype_name: self.dtype_name,
            total_cells: self.total_cells,
            valued_cells: self.valued_cells,
            fill_cells: self.total_cells.saturating_sub(self.valued_cells),
            min: if self.valued_cells == 0 {
                0.0
            } else {
                self.min
            },
            max: if self.valued_cells == 0 {
                0.0
            } else {
                self.max
            },
            sum: self.sum,
            sum_sq: self.sum_sq,
        }
    }
}

fn write_band_stats(f: &mut fmt::Formatter<'_>, band: &BandStats) -> fmt::Result {
    writeln!(
        f,
        "  ── Band {} ({}) ─────────────────────────────────",
        band.band_index, band.dtype_name
    )?;
    writeln!(f)?;
    writeln!(
        f,
        "    Total cells    : {:>12}",
        format_count(band.total_cells)
    )?;
    writeln!(
        f,
        "    Valued cells   : {:>12}  ({:.1}%)",
        format_count(band.valued_cells),
        band.valued_percentage()
    )?;
    writeln!(
        f,
        "    NoData cells   : {:>12}  ({:.1}%)",
        format_count(band.fill_cells),
        band.fill_percentage()
    )?;
    writeln!(f)?;

    if band.valued_cells > 0 {
        writeln!(
            f,
            "    Min            : {:>12}",
            format_stat_value(band.min)
        )?;
        writeln!(
            f,
            "    Max            : {:>12}",
            format_stat_value(band.max)
        )?;
        writeln!(
            f,
            "    Mean           : {:>12}",
            format_stat_value(band.mean())
        )?;
        writeln!(
            f,
            "    Std dev        : {:>12}",
            format_stat_value(band.stddev())
        )?;
        writeln!(f)?;
    } else {
        writeln!(f, "    (no valued cells)")?;
        writeln!(f)?;
    }

    Ok(())
}

impl fmt::Display for SourceRasterReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "╔══════════════════════════════════════════════════════════╗"
        )?;
        writeln!(
            f,
            "║             Source GeoTIFF Statistics Report            ║"
        )?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════╝"
        )?;
        writeln!(f)?;
        writeln!(f, "  Dimensions     : {} × {} px", self.width, self.height)?;
        writeln!(f, "  Total pixels   : {}", format_count(self.total_pixels))?;
        writeln!(f, "  Bands          : {}", self.bands.len())?;
        writeln!(f)?;

        for band in &self.bands {
            write_band_stats(f, band)?;
        }

        Ok(())
    }
}

impl fmt::Display for ConversionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "╔══════════════════════════════════════════════════════════╗"
        )?;
        writeln!(
            f,
            "║              Conversion Statistics Report               ║"
        )?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════╝"
        )?;
        writeln!(f)?;
        writeln!(f, "  Refinement level : {}", self.refinement_level)?;
        writeln!(f, "  Chunk level      : {}", self.chunk_level)?;
        writeln!(f, "  Chunks           : {}", self.num_chunks)?;
        writeln!(f, "  Chunk size       : {} cells", self.chunk_size)?;
        writeln!(f)?;

        for band in &self.bands {
            write_band_stats(f, band)?;
        }

        Ok(())
    }
}

fn format_count(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

fn format_stat_value(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{:.0}", v)
    } else {
        format!("{:.6}", v)
    }
}
