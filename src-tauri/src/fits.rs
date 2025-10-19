use anyhow::{ensure, Result};
use fitsio::{hdu::HduInfo, FitsFile};

pub struct FitsImage {
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
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

    Ok(FitsImage {
        data,
        width: w,
        height: h,
    })
}
