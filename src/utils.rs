// Copyright 2021-2022 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate dssim_core;
extern crate imgref;
extern crate load_image;

use std::path::Path;

use anyhow::Result;
use dssim_core::*;
use imgref::*;
use load_image::*;

// Copied from https://github.com/kornelski/dssim/blob/main/src/lib.rs

/// Load PNG or JPEG image from the given path. Applies color profiles and converts to sRGB.
pub fn load_image(
    attr: &Dssim,
    path: impl AsRef<Path>,
) -> Result<DssimImage<f32>, lodepng::Error> {
    load(attr, path.as_ref())
}

fn load(attr: &Dssim, path: &Path) -> Result<DssimImage<f32>, lodepng::Error> {
    let img = load_image::load_path(path)?;
    Ok(match img.bitmap {
        ImageData::RGB8(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgblu(),
            img.width,
            img.height,
        )),
        ImageData::RGB16(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgblu(),
            img.width,
            img.height,
        )),
        ImageData::RGBA8(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgbaplu(),
            img.width,
            img.height,
        )),
        ImageData::RGBA16(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgbaplu(),
            img.width,
            img.height,
        )),
        ImageData::GRAY8(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgblu(),
            img.width,
            img.height,
        )),
        ImageData::GRAY16(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgblu(),
            img.width,
            img.height,
        )),
        ImageData::GRAYA8(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgbaplu(),
            img.width,
            img.height,
        )),
        ImageData::GRAYA16(ref bitmap) => attr.create_image(&Img::new(
            bitmap.to_rgbaplu(),
            img.width,
            img.height,
        )),
    }
    .expect("infallible"))
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

        let strip_jpg =
            load_image(&attr, "tests/profile-stripped.jpg").unwrap();
        let (diff, _) = attr.compare(&strip_jpg, prof_jpg);
        assert!(diff > 0.008, "{}", diff);

        let strip_png =
            load_image(&attr, "tests/profile-stripped.png").unwrap();
        let (diff, _) = attr.compare(&strip_jpg, strip_png);

        assert!(diff > 0.009, "{}", diff);
    }
}
