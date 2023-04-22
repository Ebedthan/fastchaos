// Copyright 2021-2023 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate itertools;
extern crate noodles;
extern crate plotters;
extern crate rayon;
extern crate serde;

use std::io::{self, BufReader, Write};
use std::path::PathBuf;
use std::process;
use std::str;

use anyhow::Result;
use itertools::Itertools;
use noodles::fasta;
use plotters::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

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
    fn draw(&self, outdir: &PathBuf) -> Result<()> {
        let png = format!("{}_cgr.png", self.id);
        let mut opath = PathBuf::from(outdir);
        opath.push(png);

        let root_area =
            BitMapBackend::new(&opath, (1024, 768)).into_drawing_area();

        root_area.fill(&WHITE).unwrap();

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(-1f64..1f64, -1f64..1f64)
            .unwrap();
        ctx.configure_mesh().draw().unwrap();

        ctx.draw_series(self.cgrs.iter().map(|point| {
            Circle::new(*point, 2, ShapeStyle::from(&BLACK).filled())
        }))
        .unwrap();

        Ok(())
    }
}

trait DnaToChaos {
    fn record_to_chaos(&self) -> Chaos;
}

impl DnaToChaos for fasta::Record {
    fn record_to_chaos(&self) -> Chaos {
        let mut result: Vec<(f64, f64)> = Vec::new();

        let an = vec![1.00, 1.00];
        let tn = vec![-1.00, 1.00];
        let cn = vec![-1.00, -1.00];
        let gn = vec![1.00, -1.00];
        let mut aa;
        let mut bb;

        for (index, nucleotide) in self.sequence().as_ref().iter().enumerate() {
            if index == 0 {
                if *nucleotide == b'A' {
                    aa = an[0] * 0.5;
                    bb = an[1] * 0.5;
                } else if *nucleotide == b'T' {
                    aa = tn[0] * 0.5;
                    bb = tn[1] * 0.5;
                } else if *nucleotide == b'C' {
                    aa = cn[0] * 0.5;
                    bb = cn[1] * 0.5;
                } else {
                    aa = gn[0] * 0.5;
                    bb = gn[1] * 0.5;
                }
            } else if *nucleotide == b'A' {
                aa = 0.5 * (result[index - 1].0 + an[0]);
                bb = 0.5 * (result[index - 1].1 + an[1]);
            } else if *nucleotide == b'T' {
                aa = 0.5 * (result[index - 1].0 + tn[0]);
                bb = 0.5 * (result[index - 1].1 + tn[1]);
            } else if *nucleotide == b'C' {
                aa = 0.5 * (result[index - 1].0 + cn[0]);
                bb = 0.5 * (result[index - 1].1 + cn[1]);
            } else {
                aa = 0.5 * (result[index - 1].0 + gn[0]);
                bb = 0.5 * (result[index - 1].1 + gn[1]);
            }

            result.push((aa, bb));
        }

        Chaos {
            id: self.name().to_string(),
            cgrs: result,
        }
    }
}

pub fn draw<R: io::Read>(source: R, destination: PathBuf) -> Result<()> {
    let mut reader = fasta::Reader::new(BufReader::new(source));

    for result in reader.records() {
        // Unwrap record
        let record = result?;

        // Convert record to chaos
        let chaos = record.record_to_chaos();

        // Draw CGR of chao
        chaos.draw(&destination)?;
    }

    Ok(())
}

pub fn compare_images(images: Vec<String>) -> Vec<(String, String, f64)> {
    let mut result = Vec::new();
    let attr = dssim_core::Dssim::new();

    let files = images
        .par_iter()
        .map(|file| -> Result<_, String> {
            let image = utils::load_image(&attr, file)
                .map_err(|e| format!("Cannot load {}, because: {}", file, e))?;
            Ok((file, image))
        })
        .collect::<Result<Vec<_>, _>>();

    let files = match files {
        Ok(f) => f,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let it = files.into_iter().combinations_with_replacement(2);

    for combination in it {
        if combination[0].1.width() != combination[1].1.width()
            || combination[0].1.height() != combination[1].1.height()
        {
            writeln!(
                io::stderr(),
                "Image {} has a different size ({}x{}) than {} ({}x{})\n",
                combination[0].0,
                combination[0].1.width(),
                combination[0].1.height(),
                combination[1].0,
                combination[1].1.width(),
                combination[1].1.height()
            )
            .expect("Cannot write to stderr");
            process::exit(1);
        }

        let (dssim, _) = attr.compare(&combination[0].1, &combination[1].1);
        result.push((
            combination[0].0.to_string(),
            combination[1].0.to_string(),
            f64::from(dssim),
        ));
    }

    result
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

        let mut ot = PathBuf::from(odir);
        std::fs::create_dir(&ot).unwrap();
        ot.push("sq0_cgr.png");

        chaos.draw(&PathBuf::from(odir)).unwrap();
        let gr = compare_images(vec![ot.to_str().unwrap().to_string()]);

        assert_eq!(
            gr,
            vec![(
                ot.to_str().unwrap().to_string(),
                ot.to_str().unwrap().to_string(),
                0_f64
            )]
        );

        fs::remove_dir_all(&ot.parent().unwrap()).unwrap();
    }
}
