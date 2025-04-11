// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use rayon::iter::ParallelIterator;
use std::fmt;
use std::io::{self, BufReader};
use std::str;

use anyhow::Result;
use noodles::fasta;
use rayon::iter::IntoParallelIterator;
use serde::{Deserialize, Serialize};

/// Integer Chaos Game Representation (ICGR) for a sequence
/// Using Strings for serialization compatibility.
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

/// Representation of a DNA sequence using the Integer Chaos Game Representation (ICGR) method.
///
/// This struct represents a DNA sequence using the Integer Chaos Game Representation (ICGR) method.
/// It contains a vector of ICGR which represent the whole DNA sequence.
///
/// # Examples
///
/// ```
/// use fastchaos::icgr::IChaos;
///
/// let icgr = IChaos::new("seq1", "description", vec![1, 2, 3]);
/// assert_eq!(icgr.id(), "seq1");
/// assert_eq!(icgr.desc(), Some("description"));
/// assert_eq!(icgr.icgrs(), &[1, 2, 3]);
/// ```
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
    /// Computes the total length of the DNA sequence represented by the ICGR.
    fn length(&self) -> usize {
        self.icgrs.iter().map(|x| x.n).sum()
    }

    /// Converts the ICGR to a FASTA record.
    fn to_fasta(&self) -> fasta::Record {
        let sequence = self.decode_icgr();
        fasta::Record::new(
            fasta::record::Definition::new(&self.id, self.desc.clone()),
            fasta::record::Sequence::from(sequence),
        )
    }

    /// Serialize the IChaos object to a string
    ///
    /// The format is:
    /// `SeqID\tx1,y1,n1;x2,y2,n2;...`
    fn to_text(&self) -> String {
        let vec_str = self
            .icgrs
            .iter()
            .map(|x| format!("{},{},{}", x.x, x.y, x.n))
            .collect::<Vec<_>>()
            .join(";");

        format!("{}\t{}\n", self.id, vec_str)
    }

    /// Decodes ICGR values back into a nucleotide sequence.
    fn decode_icgr(&self) -> Vec<u8> {
        let mut complete_dna = Vec::with_capacity(self.length());
        let base: i128 = 2;

        for icgr in &self.icgrs {
            let mut an = vec![0; icgr.n];
            let mut bn = vec![0; icgr.n];
            an[icgr.n - 1] = icgr.x.parse().unwrap_or(0);
            bn[icgr.n - 1] = icgr.y.parse().unwrap_or(0);

            let mut seq = Vec::with_capacity(icgr.n);

            for index in (0..icgr.n).rev() {
                let nucleotide = get_nucleotide(an[index], bn[index]).unwrap_or('N');
                seq.push(nucleotide);
                if index > 0 {
                    let (f, g) = get_cgr_vertex(an[index], bn[index]).unwrap_or((0, 0));
                    an[index - 1] = an[index] - base.pow(index as u32) * f;
                    bn[index - 1] = bn[index] - base.pow(index as u32) * g;
                }
            }
            seq.reverse();
            complete_dna.extend(seq.into_iter().map(|c| c as u8));
        }
        complete_dna
    }
}

impl Icgr {
    fn from_sequence(sequence: &[u8]) -> Vec<Icgr> {
        let seq = String::from_utf8_lossy(sequence);
        let seq_length = seq.len();

        let chunks: Vec<&str> = if seq_length > 100 {
            str_chunks(&seq, 100).collect()
        } else {
            vec![&seq]
        };

        // Parallelize for large datasets
        chunks.into_par_iter().map(Self::icgr_from_chunk).collect()
    }

    fn icgr_from_chunk(chunk: &str) -> Icgr {
        let an = [1, 1];
        let tn = [-1, 1];
        let cn = [-1, -1];
        let gn = [1, -1];
        let base: i128 = 2;

        let mut xx = Vec::with_capacity(chunk.len());
        let mut yy = Vec::with_capacity(chunk.len());

        for (index, nucleotide) in chunk.chars().enumerate() {
            let new_index = index as u32;
            let (aa, bb) = match index {
                0 => match nucleotide {
                    'A' => (an[0], an[1]),
                    'T' => (tn[0], tn[1]),
                    'C' => (cn[0], cn[1]),
                    'G' => (gn[0], gn[1]),
                    _ => (0, 0),
                },
                _ => {
                    let (prev_x, prev_y) = (xx[index - 1], yy[index - 1]);
                    let power = base.pow(new_index);
                    match nucleotide {
                        'A' => (prev_x + power, prev_y + power),
                        'T' => (prev_x - power, prev_y + power),
                        'C' => (prev_x - power, prev_y - power),
                        'G' => (prev_x - power, prev_y - power),
                        _ => (prev_x, prev_y),
                    }
                }
            };
            xx.push(aa);
            yy.push(bb);
        }
        let n = chunk.len();
        Icgr {
            x: xx[n - 1].to_string(),
            y: yy[n - 1].to_string(),
            n,
        }
    }
}

/// Converts a FASTA record into IChaos format.
fn from_record(record: fasta::Record) -> IChaos {
    IChaos {
        id: record.name().to_string(),
        desc: record.description().map(str::to_string),
        icgrs: Icgr::from_sequence(record.sequence().as_ref()),
    }
}

/// Determines the nucleotide from ICGR coordinates.
fn get_nucleotide(x: i128, y: i128) -> Result<char> {
    match (x.signum(), y.signum()) {
        (1, 1) => Ok('A'),
        (1, -1) => Ok('G'),
        (-1, 1) => Ok('T'),
        (-1, -1) => Ok('C'),
        _ => Ok('N'),
    }
}

/// Determines the CGR vertex from ICGR coordinates.
fn get_cgr_vertex(x: i128, y: i128) -> Result<(i128, i128)> {
    match (x.signum(), y.signum()) {
        (1, 1) => Ok((1, 1)),
        (1, -1) => Ok((1, -1)),
        (-1, 1) => Ok((-1, 1)),
        (-1, -1) => Ok((-1, -1)),
        _ => Ok((0, 0)),
    }
}

/// Function generating an iterator of chunks of sequence
#[inline]
fn str_chunks<'a>(s: &'a str, n: usize) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(s.as_bytes().chunks(n).map(|c| str::from_utf8(c).unwrap()))
}

fn string_to_ichaos(s: &str) -> Result<IChaos, Box<dyn std::error::Error>> {
    let mut parts = s.trim().split('\t');

    let id = parts
        .next()
        .ok_or_else(|| -> Box<dyn std::error::Error> { "Missing sequence ID".into() })?
        .to_string();

    let data = parts
        .next()
        .ok_or_else(|| -> Box<dyn std::error::Error> { "Missing data after ID".into() })?;

    let icgrs = data
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|triple| {
            let coords: Vec<&str> = triple.split(',').collect();
            if coords.len() != 3 {
                return Err(format!("Invalid triplet format: {}", triple).into());
            }

            Ok(Icgr {
                x: coords[0].parse()?,
                y: coords[1].parse()?,
                n: coords[2].parse()?,
            })
        })
        .collect::<Result<Vec<Icgr>, Box<dyn std::error::Error>>>()?;

    Ok(IChaos {
        id,
        desc: None,
        icgrs,
    })
}

pub fn encode<W: io::Write>(source: String, mut destination: W) -> Result<()> {
    let mut reader = fasta::Reader::new(BufReader::new(source.as_bytes()));
    for result in reader.records() {
        let record = result?;
        let ichaos = from_record(record);
        let text = ichaos.to_text();

        // Also write to destination file if provided
        destination.write_all(text.as_bytes())?;
    }

    Ok(())
}

pub fn decode<W: io::Write>(source: String, mut destination: W) -> Result<()> {
    for line in source.lines() {
        // Unwrapping to get ichaos
        let ichaos = string_to_ichaos(line).unwrap();

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
                desc: None,
                icgrs: vec![Icgr {
                    x: "515".to_string(),
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
                x: "515".to_string(),
                y: "783".to_string(),
                n: 10,
            }],
        };

        assert_eq!(ichaos.length(), 10);
        assert_eq!(
            ichaos.to_fasta(),
            fasta::Record::new(
                fasta::record::Definition::new("sq0", Some("".to_string())),
                fasta::record::Sequence::from(b"ATTCCCCTAA".to_vec()),
            )
        );
        assert_eq!(ichaos.decode_icgr(), b"ATTCCCCTAA".to_vec());
    }

    #[test]
    fn test_from_sequence() {
        let ichaos = IChaos {
            id: "sq0".to_string(),
            desc: Some(String::from("")),
            icgrs: vec![Icgr {
                x: "515".to_string(),
                y: "783".to_string(),
                n: 10,
            }],
        };

        assert_eq!(Icgr::from_sequence(b"ATTGCCGTAA"), ichaos.icgrs);
    }
    /*
    #[test]
    fn test_encode_decode() {
        let file = std::fs::File::open("tests/homo_sapiens_mitochondrion.fa").unwrap();

        let destination = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("homo.icgr")
            .unwrap();

        assert!(encode(&file, &destination).is_ok());
        assert!(decode(std::fs::File::open("homo.icgr").unwrap(), io::stdout()).is_ok());

        std::fs::remove_file("homo.icgr").unwrap();
    }*/
}
