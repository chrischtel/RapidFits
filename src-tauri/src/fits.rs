use anyhow::{ensure, Result};
use fitsio::{hdu::HduInfo, FitsFile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStats {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub stddev: f32,
    pub median: f32,
    pub histogram: Vec<u32>, // 256 bins
}

pub struct FitsImage {
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub stats: ImageStats,
}

pub fn load_fits_f32(path: &str) -> Result<FitsImage> {
    let mut f = FitsFile::open(path)?;
    let hdu = f.primary_hdu()?;

    // Safely match and borrow shape info
    let shape = match &hdu.info {
        HduInfo::ImageInfo { shape, .. } => shape,
        _ => anyhow::bail!("Primary HDU is not an image"),
    };

    ensure!(shape.len() == 2, "expected 2D image");

    let (h, w) = (shape[0] as usize, shape[1] as usize);

    let data: Vec<f32> = hdu.read_image(&mut f)?;

    // Calculate statistics
    let stats = calculate_statistics(&data);

    Ok(FitsImage {
        data,
        width: w,
        height: h,
        stats,
    })
}

fn calculate_statistics(data: &[f32]) -> ImageStats {
    // Filter out NaN and infinite values
    let valid_data: Vec<f32> = data.iter().copied().filter(|&x| x.is_finite()).collect();

    if valid_data.is_empty() {
        return ImageStats {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            stddev: 0.0,
            median: 0.0,
            histogram: vec![0; 256],
        };
    }

    // Min and max
    let min = *valid_data
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max = *valid_data
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    // Mean
    let sum: f32 = valid_data.iter().sum();
    let mean = sum / valid_data.len() as f32;

    // Standard deviation
    let variance: f32 =
        valid_data.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / valid_data.len() as f32;
    let stddev = variance.sqrt();

    // Median (approximate with sorting)
    let mut sorted = valid_data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];

    // Histogram (256 bins)
    let mut histogram = vec![0u32; 256];
    let range = max - min;
    if range > 0.0 {
        for &value in &valid_data {
            let normalized = ((value - min) / range).clamp(0.0, 1.0);
            let bin = (normalized * 255.0) as usize;
            histogram[bin] += 1;
        }
    }

    ImageStats {
        min,
        max,
        mean,
        stddev,
        median,
        histogram,
    }
}

/// Calculate percentile-based stretch limits for auto-stretch
pub fn calculate_auto_stretch(
    stats: &ImageStats,
    data: &[f32],
    low_percentile: f32,
    high_percentile: f32,
) -> (f32, f32) {
    let mut sorted: Vec<f32> = data.iter().copied().filter(|&x| x.is_finite()).collect();

    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if sorted.is_empty() {
        return (stats.min, stats.max);
    }

    let low_idx = ((sorted.len() as f32 * low_percentile / 100.0) as usize).min(sorted.len() - 1);
    let high_idx = ((sorted.len() as f32 * high_percentile / 100.0) as usize).min(sorted.len() - 1);

    (sorted[low_idx], sorted[high_idx])
}
