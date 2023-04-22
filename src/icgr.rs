// Copyright 2021-2023 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate noodles;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate zstd;

use std::fmt;
use std::io::{self, BufReader};
use std::str;

use anyhow::Result;
use noodles::fasta;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use zstd::stream;

/// The Integer Chaos Game Representation Format ------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IChaos {
    /// A DNA sequence ID: all characters before first whitespace in sequence header
    id: String,

    /// A DNA sequence description: all characters after first whitespace
    desc: Option<String>,

    /// A vector of ICGR which represent the whole DNA sequence
    icgrs: Vec<Icgr>,
}

impl IChaos {
    #[inline]
    fn length(&self) -> usize {
        self.icgrs.par_iter().map(|x| x.n).sum()
    }

    fn to_fasta(&self) -> fasta::Record {
        fasta::Record::new(
            fasta::record::Definition::new(
                &self.id,
                Some(self.desc.as_ref().unwrap().to_string()),
            ),
            fasta::record::Sequence::from(self.clone().decode_icgr()),
        )
    }

    #[inline]
    fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Cannot convert IChaos to JSON")
    }

    fn decode_icgr(&self) -> Vec<u8> {
        let mut complete_dna = Vec::with_capacity(self.length());

        for icgr in &self.icgrs {
            let mut an: Vec<i128> = vec![0; icgr.n];
            let mut bn: Vec<i128> = vec![0; icgr.n];
            an[icgr.n - 1] = icgr.x.parse::<i128>().unwrap();
            bn[icgr.n - 1] = icgr.y.parse::<i128>().unwrap();
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

/// Integer Chaos Game Representation (ICGR) for a sequence
/// Not all platforms supports 128 bits integers along
/// with Rust versions before 1.26
/// Moreover, serde_json does not yet support i128
/// https://github.com/serde-rs/json/issues/846
/// So the integers will be stored as String and converted as needed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Icgr {
    /// First integer of the ICGR
    x: String,

    /// Second integer of the ICGR
    y: String,

    /// Sequence length
    n: usize,
}

impl fmt::Display for Icgr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{},{}]", self.x, self.y, self.n)
    }
}

impl Icgr {
    fn from_sequence(sequence: &[u8]) -> Vec<Icgr> {
        let mut icgrs = Vec::new();

        let an = vec![1, 1];
        let tn = vec![-1, 1];
        let cn = vec![-1, -1];
        let gn = vec![1, -1];
        let base: i128 = 2;

        // Get sequence length
        let seq_length = sequence.len();
        let seq = String::from_utf8_lossy(sequence);

        // ICGR work for sequence with max length of 100 due to
        // exponentiation of 2 which easily overflow after a certain number
        if seq_length > 100 {
            // TODO: try to paralelize this loop
            for chunk in str_chunks(&seq, 100) {
                let mut aa;
                let mut bb;
                let mut xx: Vec<i128> = Vec::new();
                let mut yy: Vec<i128> = Vec::new();
                let chunk_length = chunk.len();
                // TODO: try to paralelize this loop
                for (index, nucleotide) in chunk.chars().enumerate() {
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

                icgrs.push(Icgr {
                    x: xx[chunk_length - 1].to_string(),
                    y: yy[chunk_length - 1].to_string(),
                    n: chunk_length,
                });
            }
        } else {
            let mut aa;
            let mut bb;
            let mut xx: Vec<i128> = Vec::new();
            let mut yy: Vec<i128> = Vec::new();

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

            icgrs.push(Icgr {
                x: xx[seq_length - 1].to_string(),
                y: yy[seq_length - 1].to_string(),
                n: seq_length,
            });
        }

        icgrs
    }
}

fn from_record(record: fasta::Record) -> IChaos {
    IChaos {
        id: record.name().to_string(),
        desc: Some(record.description().unwrap_or("").to_string()),
        icgrs: Icgr::from_sequence(record.sequence().as_ref()),
    }
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

/// Function generating an iterator of chunks of sequence
#[inline]
fn str_chunks<'a>(
    s: &'a str,
    n: usize,
) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(s.as_bytes().chunks(n).map(|c| str::from_utf8(c).unwrap()))
}

pub fn encode<R: io::Read, W: io::Write>(
    source: R,
    mut destination: W,
) -> Result<()> {
    // Openning stream using noodle fasta reader
    let mut reader = fasta::Reader::new(BufReader::new(source));

    // Iterating through all records
    for result in reader.records() {
        // Unwraping to get record
        let record = result?;

        // Convert record to IChaos
        let ichaos = from_record(record);

        // Convert Ichaos to a JSON string
        let json = ichaos.to_json();

        // Writing JSON as compressed zstd stream to destination
        stream::copy_encode(json.as_bytes(), destination.by_ref(), 9)?;
    }

    Ok(())
}

pub fn decode<R: io::Read, W: io::Write>(
    source: R,
    mut destination: W,
) -> Result<()> {
    // Decompress stream with zstd decompress
    let stream = serde_json::Deserializer::from_reader(
        stream::read::Decoder::new(source)?,
    )
    .into_iter::<IChaos>();

    for result in stream {
        // Unwrapping to get ichaos
        let ichaos = result?;

        // Convert to fasta record
        let record = ichaos.to_fasta();

        destination.write_all(
            format!(
                ">{} {}\n{}",
                record.name(),
                record.description().unwrap_or(""),
                String::from_utf8_lossy(record.sequence().as_ref())
            )
            .as_bytes(),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_nucleotide() {
        assert_eq!(get_nucleotide(1_i128, 1_i128).unwrap(), 'A');
        assert_eq!(get_nucleotide(1_i128, -1_i128).unwrap(), 'G');
        assert_eq!(get_nucleotide(-1_i128, -1_i128).unwrap(), 'C');
        assert_eq!(get_nucleotide(-1_i128, 1_i128).unwrap(), 'T');
        assert_eq!(get_nucleotide(0_i128, 0_i128).unwrap(), 'N');
    }

    #[test]
    fn test_get_cgr_vertex() {
        assert_eq!(get_cgr_vertex(1_i128, 1_i128).unwrap(), (1, 1));
        assert_eq!(get_cgr_vertex(1_i128, -1_i128).unwrap(), (1, -1));
        assert_eq!(get_cgr_vertex(-1_i128, 1_i128).unwrap(), (-1, 1));
        assert_eq!(get_cgr_vertex(-1_i128, -1_i128).unwrap(), (-1, -1));
        assert_eq!(get_cgr_vertex(0_i128, 0_i128).unwrap(), (0, 0));
    }

    #[test]
    fn test_from_record() {
        let seq = fasta::Record::new(
            fasta::record::Definition::new("sq0", None),
            fasta::record::Sequence::from(b"ATTGCCGTAA".to_vec()),
        );

        assert_eq!(
            from_record(seq),
            IChaos {
                id: "sq0".to_string(),
                desc: Some(String::from("")),
                icgrs: vec![Icgr {
                    x: "659".to_string(),
                    y: "783".to_string(),
                    n: 10
                }]
            }
        );
    }

    #[test]
    fn test_ichaos() {
        let ichaos = IChaos {
            id: "sq0".to_string(),
            desc: Some(String::from("")),
            icgrs: vec![Icgr {
                x: "659".to_string(),
                y: "783".to_string(),
                n: 10,
            }],
        };

        assert_eq!(ichaos.length(), 10);
        assert_eq!(
            ichaos.to_fasta(),
            fasta::Record::new(
                fasta::record::Definition::new("sq0", Some("".to_string())),
                fasta::record::Sequence::from(b"ATTGCCGTAA".to_vec()),
            )
        );
        assert_eq!(
            ichaos.to_json(),
            "{\"id\":\"sq0\",\"desc\":\"\",\"icgrs\":[{\"x\":\"659\",\"y\":\"783\",\"n\":10}]}".to_string()
        );

        assert_eq!(ichaos.decode_icgr(), b"ATTGCCGTAA".to_vec());
    }

    #[test]
    fn test_from_sequence() {
        let ichaos = IChaos {
            id: "sq0".to_string(),
            desc: Some(String::from("")),
            icgrs: vec![Icgr {
                x: "659".to_string(),
                y: "783".to_string(),
                n: 10,
            }],
        };

        assert_eq!(Icgr::from_sequence(b"ATTGCCGTAA"), ichaos.icgrs);
    }

    #[test]
    fn test_encode_decode() {
        let file =
            std::fs::File::open("tests/homo_sapiens_mitochondrion.fa").unwrap();

        let destination = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("homo.icgr")
            .unwrap();

        assert!(encode(&file, &destination).is_ok());
        assert!(decode(
            std::fs::File::open("homo.icgr").unwrap(),
            io::stdout()
        )
        .is_ok());

        std::fs::remove_file("homo.icgr").unwrap();
    }
}
