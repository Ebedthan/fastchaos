// Copyright 2021-2022 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate bincode;
extern crate bio;
extern crate dssim_core;
extern crate itertools;
extern crate rayon;
extern crate serde;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::str;

use anyhow::Result;
use bio::io::fasta;
use dssim_core::*;
use imgref::*;
use itertools::Itertools;
use load_image::*;
use plotters::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// The Chaos Game Representation Format --------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chaos {
    /// A DNA sequence ID: all characters before first whitespace in sequence header
    id: String,

    /// A vector of CGR for drawing and comparison
    cgrs: Vec<(f64, f64)>,
}

impl Chaos {
    fn draw(&self, outdir: &str) -> Result<()> {
        let mut opath = PathBuf::from(outdir);
        let png = format!("{}_cgr.png", self.id);
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

/// The Integer Chaos Game Representation Format ------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IChaos {
    /// A DNA sequence ID: all characters before first whitespace in sequence header
    id: String,

    /// A vector of ICGR which represent the whole DNA sequence
    icgrs: Vec<Icgr>,
}

/// Integer Chaos Game Representation (ICGR) for a sequence
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Icgr {
    /// First integer of the ICGR
    x: i128,

    /// Second integer of the ICGR
    y: i128,

    /// Sequence length
    n: usize,
}

impl IChaos {
    fn length(&mut self) -> usize {
        self.icgrs.par_iter().map(|x| x.n).sum()
    }

    fn to_fasta(&self) -> fasta::Record {
        fasta::Record::with_attrs(
            &self.id,
            Some(""),
            &self.clone().decode_icgr(),
        )
    }

    /// Returns a DNA string from a supplied ICGR
    ///
    /// # Arguments
    ///
    /// * `cgrs` - A vector of ICGRs
    ///
    /// # Examples
    ///
    /// ```
    /// let icgr: Vec<ICGR> = Vec::new();
    /// icgr.push(encode_dna("ATCT"));
    ///
    /// let dna = decode_icgr(icgr).unwrap();
    ///
    ///
    fn decode_icgr(&mut self) -> Vec<u8> {
        let mut complete_dna = Vec::with_capacity(self.length());

        for icgr in &self.icgrs {
            let mut an: Vec<i128> = vec![0; icgr.n];
            let mut bn: Vec<i128> = vec![0; icgr.n];
            an[icgr.n - 1] = icgr.x;
            bn[icgr.n - 1] = icgr.y;
            let base: i128 = 2;

            let mut seq = Vec::with_capacity(icgr.n);

            for index in (0..icgr.n).step_by(1).rev() {
                let nucleotide = get_nucleotide(an[index], bn[index]).unwrap();
                seq.push(nucleotide);
                let (f, g) = get_cgr_vertex(an[index], bn[index]).unwrap();
                if index != 0 {
                    an[index - 1] = an[index] - base.pow(index as u32) * f;
                    bn[index - 1] = bn[index] - base.pow(index as u32) * g;
                }
            }
            seq.reverse();
            let merged: Vec<u8> =
                seq.par_iter().map(|c| *c as u8).collect::<Vec<_>>();

            complete_dna.extend(merged);
        }

        complete_dna
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

        for (index, nucleotide) in self.seq().iter().enumerate() {
            if index == 0 {
                if *nucleotide == b'A' {
                    aa = an[0] as f64 * 0.5;
                    bb = an[1] as f64 * 0.5;
                } else if *nucleotide == b'T' {
                    aa = tn[0] as f64 * 0.5;
                    bb = tn[1] as f64 * 0.5;
                } else if *nucleotide == b'C' {
                    aa = cn[0] as f64 * 0.5;
                    bb = cn[1] as f64 * 0.5;
                } else {
                    aa = gn[0] as f64 * 0.5;
                    bb = gn[1] as f64 * 0.5;
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
            id: self.id().to_string(),
            cgrs: result,
        }
    }
}

/// Returns an ICGR for a DNA string
///
/// # Arguments
///
/// * `seq` - A DNA String
///
/// # Examples
///
/// ```
/// let icgr: Icgr = encode_dna("ATCG").unwrap();
/// ```
///
pub fn dna_to_icgr(seq: &str) -> Result<Icgr> {
    let an = vec![1, 1];
    let tn = vec![-1, 1];
    let cn = vec![-1, -1];
    let gn = vec![1, -1];
    let mut aa;
    let mut bb;
    let mut xx: Vec<i128> = Vec::new();
    let mut yy: Vec<i128> = Vec::new();
    let base: i128 = 2;

    // Get sequence length
    let seq_length = seq.len();

    for (index, nucleotide) in seq.chars().enumerate() {
        if index == 0 {
            if nucleotide == 'A' {
                aa = an[0];
                bb = an[1];
            } else if nucleotide == 'T' {
                aa = tn[0];
                bb = tn[1];
            } else if nucleotide == 'C' {
                aa = cn[0];
                bb = cn[1];
            } else {
                aa = gn[0];
                bb = gn[1];
            }
        } else {
            let new_index = index as u32;
            if nucleotide == 'A' {
                aa = xx[index - 1] + base.pow(new_index);
                bb = yy[index - 1] + base.pow(new_index);
            } else if nucleotide == 'T' {
                aa = xx[index - 1] - base.pow(new_index);
                bb = yy[index - 1] + base.pow(new_index);
            } else if nucleotide == 'C' {
                aa = xx[index - 1] - base.pow(new_index);
                bb = yy[index - 1] - base.pow(new_index);
            } else {
                aa = xx[index - 1] + base.pow(new_index);
                bb = yy[index - 1] - base.pow(new_index);
            }
        }

        xx.push(aa);
        yy.push(bb);
    }

    let icgr = Icgr {
        x: xx[seq_length - 1],
        y: yy[seq_length - 1],
        n: seq_length,
    };

    Ok(icgr)
}

fn get_nucleotide(x: i128, y: i128) -> Result<char> {
    if x > 0 && y > 0 {
        Ok('A')
    } else if x > 0 && y < 0 {
        Ok('G')
    } else if x < 0 && y > 0 {
        Ok('T')
    } else if x < 0 && y < 0 {
        Ok('C')
    } else {
        Ok('N')
    }
}

fn get_cgr_vertex(x: i128, y: i128) -> Result<(i128, i128)> {
    if x > 0 && y > 0 {
        Ok((1, 1))
    } else if x > 0 && y < 0 {
        Ok((1, -1))
    } else if x < 0 && y > 0 {
        Ok((-1, 1))
    } else if x < 0 && y < 0 {
        Ok((-1, -1))
    } else {
        Ok((0, 0))
    }
}

pub fn str_chunks<'a>(
    s: &'a str,
    n: usize,
) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(s.as_bytes().chunks(n).map(|c| str::from_utf8(c).unwrap()))
}

pub fn encode_from_file(file: &str, out: Option<&str>) -> Result<()> {
    let mut records = fasta::Reader::from_file(file)?.records();

    match out {
        Some(filename) => {
            let mut chaos: Vec<IChaos> = Vec::new();

            while let Some(Ok(record)) = records.next() {
                let mut micgrs: Vec<Icgr> = Vec::new();

                if record.seq().len() >= 100 {
                    let seq = String::from_utf8_lossy(record.seq());
                    for chunk in str_chunks(&seq, 100) {
                        micgrs.push(dna_to_icgr(chunk)?);
                    }
                } else {
                    micgrs.push(dna_to_icgr(&String::from_utf8_lossy(
                        record.seq(),
                    ))?);
                }

                chaos.push(IChaos {
                    id: record.id().to_string(),
                    icgrs: micgrs,
                });
            }

            for chao in chaos {
                bincode::serialize_into(
                    fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(filename)?,
                    &chao,
                )?;
            }

            Ok(())
        }
        None => {
            let mut chaos: Vec<IChaos> = Vec::new();

            while let Some(Ok(record)) = records.next() {
                let mut micgrs: Vec<Icgr> = Vec::new();

                if record.seq().len() >= 200 {
                    let seq = String::from_utf8_lossy(record.seq());
                    for chunk in str_chunks(&seq, 100) {
                        micgrs.push(dna_to_icgr(chunk)?);
                    }
                } else {
                    micgrs.push(dna_to_icgr(&String::from_utf8_lossy(
                        record.seq(),
                    ))?);
                }

                chaos.push(IChaos {
                    id: record.id().to_string(),
                    icgrs: micgrs,
                });
            }

            for chao in chaos {
                bincode::serialize_into(io::stdout(), &chao)?;
            }

            Ok(())
        }
    }
}

pub fn decode_from_file<P: AsRef<Path>>(
    path: P,
    out: Option<&str>,
) -> Result<()> {
    match out {
        Some(outfile) => {
            let file = fs::File::open(path)?;
            let mut writer = fasta::Writer::to_file(outfile)?;

            loop {
                match bincode::deserialize_from::<_, IChaos>(&file) {
                    Ok(v) => {
                        writer.write_record(&v.to_fasta())?;
                    }
                    Err(e) => match *e {
                        bincode::ErrorKind::Io(e)
                            if e.kind() == io::ErrorKind::UnexpectedEof =>
                        {
                            break
                        }
                        e => panic!("Error de-serializing IChaos {}", e),
                    },
                };
            }

            Ok(())
        }
        None => {
            let file = fs::File::open(path)?;
            let mut writer = fasta::Writer::new(io::stdout());

            loop {
                match bincode::deserialize_from::<_, IChaos>(&file) {
                    Ok(v) => {
                        writer.write_record(&v.to_fasta())?;
                    }
                    Err(e) => match *e {
                        bincode::ErrorKind::Io(e)
                            if e.kind() == io::ErrorKind::UnexpectedEof =>
                        {
                            break
                        }
                        e => panic!("error de-serializing IChaos {}", e),
                    },
                };
            }

            Ok(())
        }
    }
}

pub fn draw_from_file(file: &str, out: Option<&str>) -> Result<()> {
    let mut records = fasta::Reader::from_file(file)?.records();

    match out {
        Some(dirname) => {
            if !Path::new(dirname).exists() {
                fs::create_dir(dirname)?;
            }
            let mut chaos: Vec<Chaos> = Vec::new();

            while let Some(Ok(record)) = records.next() {
                chaos.push(record.record_to_chaos());
            }

            for chao in chaos {
                chao.draw(dirname).expect("Cannot draw figure");
                println!("Done drawing {}", &chao.id);
            }

            Ok(())
        }
        None => {
            let mut chaos: Vec<Chaos> = Vec::new();

            while let Some(Ok(record)) = records.next() {
                chaos.push(record.record_to_chaos());
            }

            for chao in chaos {
                chao.draw(".").expect("Cannot draw figure");
            }

            Ok(())
        }
    }
}

// Copied from https://github.com/kornelski/dssim/blob/main/src/lib.rs
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

/// Load PNG or JPEG image from the given path. Applies color profiles and converts to sRGB.
fn load_image(
    attr: &Dssim,
    path: impl AsRef<Path>,
) -> Result<DssimImage<f32>, lodepng::Error> {
    load(attr, path.as_ref())
}

pub fn compare_images(images: Vec<String>, out: Option<&str>) -> Result<()> {
    let mut res = Vec::new();
    let attr = dssim_core::Dssim::new();

    let files = images
        .iter()
        .map(|file| -> Result<_, String> {
            let image = load_image(&attr, &file)
                .map_err(|e| format!("Cannot load {}, because: {}", file, e))?;
            Ok((file, image))
        })
        .collect::<Result<Vec<_>, _>>();

    let files = match files {
        Ok(f) => f,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(exitcode::DATAERR);
        }
    };

    let it = files.into_iter().combinations_with_replacement(2);

    for combination in it {
        if combination[0].1.width() != combination[1].1.width()
            || combination[0].1.height() != combination[1].1.height()
        {
            println!(
                "Image {} has a different size ({}x{}) than {} ({}x{})\n",
                combination[0].0,
                combination[0].1.width(),
                combination[0].1.height(),
                combination[1].0,
                combination[1].1.width(),
                combination[1].1.height()
            );
        }

        let (dssim, _) = attr.compare(&combination[0].1, &combination[1].1);
        res.push((
            combination[0].0.to_string(),
            combination[1].0.to_string(),
            f64::from(dssim),
        ));
    }

    match out {
        Some(filename) => {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(filename)
                .expect("Cannot open file");
            for data in res {
                file.write_all(
                    format!("{}\t{}\t{:.8}\n", data.0, data.1, data.2)
                        .as_bytes(),
                )
                .expect("Cannot write to file");
            }
        }
        None => {
            for data in res {
                writeln!(
                    io::stdout(),
                    "{}",
                    format!("{}\t{}\t{:.8}", data.0, data.1, data.2),
                )
                .expect("Cannot write to file");
            }
        }
    }

    Ok(())
}

// Tests -------------------------------------------
#[cfg(test)]
mod tests {}
