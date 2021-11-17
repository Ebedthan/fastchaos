// Copyright 2021 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate bio;

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str;

use anyhow::Result;
use bio::io::fasta::Reader;
use serde::{Deserialize, Serialize};
use serde_json::{to_writer, to_writer_pretty};

// Define the Chaos Game Representation File Format
#[derive(Debug, Serialize, Deserialize)]
pub struct Chaos {
    id: String,
    cgrs: Vec<ICGR>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ICGR {
    x: i128,
    y: i128,
    n: usize,
}

impl Chaos {
    pub fn new() -> Self {
        Chaos {
            id: String::new(),
            cgrs: vec![ICGR::new()],
        }
    }
}

impl ICGR {
    pub fn new() -> Self {
        ICGR { x: 0, y: 0, n: 0 }
    }
}

// encode_dna encode a dna string into 3 integers by Chaos Game Representation
pub fn encode_dna(seq: &str) -> Result<ICGR> {
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
    let n = seq.len();

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

    let icgr = ICGR {
        x: xx[n - 1],
        y: yy[n - 1],
        n: n,
    };

    Ok(icgr)
}

pub fn decode_dna(icgr: ICGR) -> Result<String> {
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
    let merged: String = seq.iter().collect();

    Ok(merged)
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
    let mut records = Reader::from_file(file)?.records();

    match out {
        Some(filename) => {
            let mut chaos: Vec<Chaos> = Vec::new();

            while let Some(Ok(record)) = records.next() {
                let mut icgrs: Vec<ICGR> = Vec::new();

                if record.seq().len() >= 100 {
                    let seq = String::from_utf8_lossy(record.seq());
                    for chunk in str_chunks(&seq, 100) {
                        icgrs.push(encode_dna(chunk)?);
                    }
                    chaos.push(Chaos {
                        id: record.id().to_string(),
                        cgrs: icgrs,
                    });
                } else {
                    icgrs.push(encode_dna(&String::from_utf8_lossy(
                        record.seq(),
                    ))?);

                    chaos.push(Chaos {
                        id: record.id().to_string(),
                        cgrs: icgrs,
                    });
                }
            }

            for chao in chaos {
                to_writer(
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
            let mut chaos: Vec<Chaos> = Vec::new();

            while let Some(Ok(record)) = records.next() {
                let mut icgrs: Vec<ICGR> = Vec::new();

                if record.seq().len() >= 100 {
                    let seq = String::from_utf8_lossy(record.seq());
                    for chunk in str_chunks(&seq, 100) {
                        icgrs.push(encode_dna(chunk)?);
                    }
                    chaos.push(Chaos {
                        id: record.id().to_string(),
                        cgrs: icgrs,
                    });
                } else {
                    icgrs.push(encode_dna(&String::from_utf8_lossy(
                        record.seq(),
                    ))?);
                    chaos.push(Chaos {
                        id: record.id().to_string(),
                        cgrs: icgrs,
                    });
                }
            }

            for chao in chaos {
                to_writer_pretty(io::stdout(), &chao)?;
            }

            Ok(())
        }
    }
}

fn read_chaos_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<Chaos, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Chaos`.
    let chaos = serde_json::from_reader(reader)?;

    // Return the `Chaos`.
    Ok(chaos)
}

pub fn decode_from_file(file: &str, out: Option<&str>) -> Result<()> {
    match out {
        Some(_) => {
            let c = read_chaos_from_file(file).unwrap();
            println!("{:?}", c);

            Ok(())
        }
        None => {
            let c = read_chaos_from_file(file).unwrap();
            println!("{:?}", c);

            Ok(())
        }
    }
}

// Tests -------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // encode_dna test
    #[test]
    fn test_encode_dna() {
        let seq = "ATCGATCGATCGATCGATCG";
        assert_eq!(
            encode_dna(seq).unwrap(),
            ICGR {
                x: 209715,
                y: -629145,
                n: 20
            }
        );
    }

    // decode_dna test
    #[test]
    fn test_decode_dna() {
        assert_eq!(
            decode_dna(ICGR {
                x: 209715,
                y: -629145,
                n: 20
            })
            .unwrap(),
            "ATCGATCGATCGATCGATCG"
        );
    }
}
