// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use std::fmt;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::process;
use std::str;

use noodles::fasta;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use crate::utils;

/// The Chaos Game Representation Format --------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chaos {
    /// A DNA sequence ID: all characters before first whitespace in sequence header
    id: String,

    /// A vector of CGR for drawing and comparison
    cgrs: Vec<(f64, f64)>,
}

impl Chaos {
    /// Draws the CGR and saves it as a PNG file
    fn draw(&self, output: Option<PathBuf>) -> anyhow::Result<()> {
        let image = if let Some(out) = output {
            out
        } else {
            PathBuf::from(format!("{}.png", self.id))
        };

        let root_area = BitMapBackend::new(&image, (1024, 768)).into_drawing_area();
        root_area.fill(&WHITE)?;

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(-1f64..1f64, -1f64..1f64)?;
        ctx.configure_mesh().draw()?;

        ctx.draw_series(
            self.cgrs
                .iter()
                .map(|&point| Circle::new(point, 2, BLACK.filled())),
        )?;
        Ok(())
    }
}

/// Trait for converting DNA sequences to Chaos Game Representation (CGR)
trait DnaToChaos {
    fn record_to_chaos(&self) -> Chaos;
}

impl DnaToChaos for fasta::Record {
    fn record_to_chaos(&self) -> Chaos {
        let mut result = Vec::with_capacity(self.sequence().len());

        let nucleotides = [
            (b'A', [1.0, 1.0]),
            (b'T', [-1.0, 1.0]),
            (b'C', [-1.0, -1.0]),
            (b'G', [1.0, -1.0]),
        ];

        let mut coords = (0.0, 0.0);

        for nucleotide in self.sequence().as_ref() {
            if let Some(&(_, pos)) = nucleotides.iter().find(|&&(n, _)| n == *nucleotide) {
                coords.0 = 0.5 * (coords.0 + pos[0]);
                coords.1 = 0.5 * (coords.1 + pos[1]);
                result.push(coords);
            }
        }

        Chaos {
            id: self.name().to_string(),
            cgrs: result,
        }
    }
}

/// Reads a FASTA file, generates its CGR, and saves it as an image.
pub fn draw<R: io::Read>(source: R, destination: Option<PathBuf>) -> anyhow::Result<()> {
    let mut reader = fasta::Reader::new(BufReader::new(source));

    for result in reader.records() {
        let record = result?;
        let chaos = record.record_to_chaos();
        chaos.draw(destination.clone())?;
    }
    Ok(())
}

/// Structure to store SSIM results
#[derive(Debug)]
pub struct SSIMResult {
    query: String,
    reference: String,
    ssim: f64,
}

impl SSIMResult {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            reference: String::new(),
            ssim: 0.0,
        }
    }
    fn add(&mut self, query: String, reference: String, ssim: f64) {
        self.query = query;
        self.reference = reference;
        self.ssim = ssim;
    }
}

impl fmt::Display for SSIMResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}",
            Path::new(&self.query)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            Path::new(&self.reference)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            self.ssim
        )
    }
}

/// Compares two genome sequences based on CGR images
pub fn compare_genomes(query: &str, reference: &str) -> anyhow::Result<SSIMResult> {
    // Create temporary directory
    let dir = tempdir()?;

    let attr = dssim_core::Dssim::new();
    let mut result = SSIMResult::new();

    let qimg_out = PathBuf::from(format!("{:?}/query.png", dir.path()));
    let rimg_out = PathBuf::from(format!("{:?}/reference.png", dir.path()));
    draw(File::open(query)?, Some(qimg_out.clone()))?;
    draw(File::open(reference)?, Some(rimg_out.clone()))?;

    // Read images
    let qimage = utils::get_image(&qimg_out)?;
    let rimage = utils::get_image(&rimg_out)?;

    if utils::is_same_width_height(&qimage, &rimage) {
        let (dssim, _) = attr.compare(&qimage.0, &rimage.0);
        result.add(qimage.1, rimage.1, f64::from(dssim));
    } else {
        utils::eimgprint(&qimage, &rimage);
        process::exit(1);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_dna_to_chaos() {
        let seq = fasta::Record::new(
            fasta::record::Definition::new("sq0", None),
            fasta::record::Sequence::from(b"TAGCA".to_vec()),
        );

        assert_eq!(
            Chaos {
                id: "sq0".to_string(),
                cgrs: vec![
                    (-0.5000_f64, 0.5000_f64),
                    (0.2500_f64, 0.7500_f64),
                    (0.6250_f64, -0.1250_f64),
                    (-0.1875_f64, -0.5625_f64),
                    (0.40625_f64, 0.21875_f64)
                ]
            },
            seq.record_to_chaos()
        );
    }

    #[test]
    fn test_draw_and_compare() {
        let odir = "temp";
        let chaos = Chaos {
            id: "sq0".to_string(),
            cgrs: vec![
                (-0.5000_f64, 0.5000_f64),
                (0.2500_f64, 0.7500_f64),
                (0.6250_f64, -0.1250_f64),
                (-0.1875_f64, -0.5625_f64),
                (0.40625_f64, 0.21875_f64),
            ],
        };

        let ot = PathBuf::from(odir);
        std::fs::create_dir(&ot).unwrap();

        chaos.draw(Some(ot.clone())).unwrap();

        fs::remove_dir_all(ot).unwrap();
    }
}
