// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use dssim_core::*;
use imgref::*;
use load_image::*;

// Copied https://github.com/kornelski/dssim/blob/f3e2191efed786081f780ddea08a1e6027f31680/src/lib.rs#L10
/// Loading images
/// Load PNG or JPEG image from the given path. Applies color profiles and converts to sRGB.
pub fn load_image(attr: &Dssim, path: impl AsRef<Path>) -> Result<DssimImage<f32>, lodepng::Error> {
    load(attr, path.as_ref())
}

/// Image loading helper function.
fn load(attr: &Dssim, path: &Path) -> lodepng::Result<DssimImage<f32>, lodepng::Error> {
    let img = load_image::load_path(path).map_err(|_| lodepng::Error::new(1))?;

    Ok(match img.bitmap {
        ImageData::RGB8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::RGB16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::RGBA8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
        ImageData::RGBA16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
        ImageData::GRAY8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::GRAY16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height))
        }
        ImageData::GRAYA8(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
        ImageData::GRAYA16(ref bitmap) => {
            attr.create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height))
        }
    }
    .expect("infallible"))
}

/// Get image function
pub fn get_image(file: &PathBuf) -> anyhow::Result<(DssimImage<f32>, String)> {
    let attr = Dssim::new();
    let image = load_image(&attr, file)?;
    let filename = file
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in file path"))?;
    Ok((image, filename.to_string()))
}

// Read lines from a file
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(&filename).map_err(|e| {
        eprintln!("Error opening file {:?}: {}", filename.as_ref(), e);
        e
    })?;
    Ok(io::BufReader::new(file).lines())
}

/// Compare image dimensions
pub fn is_same_width_height(
    img1: &(DssimImage<f32>, String),
    img2: &(DssimImage<f32>, String),
) -> bool {
    img1.0.width() == img2.0.width() && img1.0.height() == img2.0.height()
}

/// Error message for size mismatch
pub fn eimgprint(img1: &(DssimImage<f32>, String), img2: &(DssimImage<f32>, String)) {
    eprintln!(
        "Image {} has a different size ({}x{}) than {} ({}x{})\n",
        img1.1,
        img1.0.width(),
        img1.0.height(),
        img2.1,
        img2.0.width(),
        img2.0.height()
    );
}

// Tests -------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_image() {
        let attr = Dssim::new();
        let prof_jpg = load_image(&attr, "tests/profile.jpg").unwrap();
        let prof_png = load_image(&attr, "tests/profile.png").unwrap();
        let (diff, _) = attr.compare(&prof_jpg, prof_png);
        assert!(diff <= 0.002);

        let strip_jpg = load_image(&attr, "tests/profile-stripped.jpg").unwrap();
        let (diff, _) = attr.compare(&strip_jpg, prof_jpg);
        assert!(diff > 0.008, "{}", diff);

        let strip_png = load_image(&attr, "tests/profile-stripped.png").unwrap();
        let (diff, _) = attr.compare(&strip_jpg, strip_png);

        assert!(diff > 0.009, "{}", diff);
    }

    #[test]
    fn image_gray() {
        let attr = Dssim::new();

        let g1 = load_image(&attr, "tests/gray1-rgba.png").unwrap();
        let g2 = load_image(&attr, "tests/gray1-pal.png").unwrap();
        let g3 = load_image(&attr, "tests/gray1-gray.png").unwrap();
        let g4 = load_image(&attr, "tests/gray1.jpg").unwrap();

        let (diff, _) = attr.compare(&g1, g2);
        assert!(diff < 0.00001);

        let (diff, _) = attr.compare(&g1, g3);
        assert!(diff < 0.00001);

        let (diff, _) = attr.compare(&g1, g4);
        assert!(diff < 0.00006);
    }

    #[test]
    fn image_gray_profile() {
        let attr = Dssim::new();

        let gp1 = load_image(&attr, "tests/gray-profile.png").unwrap();
        let gp2 = load_image(&attr, "tests/gray-profile2.png").unwrap();
        let gp3 = load_image(&attr, "tests/gray-profile.jpg").unwrap();

        let (diff, _) = attr.compare(&gp1, gp2);
        assert!(diff < 0.0003, "{}", diff);

        let (diff, _) = attr.compare(&gp1, gp3);
        assert!(diff < 0.0003, "{}", diff);
    }

    #[test]
    fn rgblu_input() {
        let ctx = Dssim::new();
        let im: ImgVec<RGBLU> = Img::new(vec![export::rgb::RGB::new(0., 0., 0.)], 1, 1);
        let imr: ImgRef<'_, RGBLU> = im.as_ref();
        ctx.create_image(&imr);
    }
}
