// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use rayon::iter::{FromParallelIterator, ParallelIterator};
use std::fmt;
use std::io::BufRead;
use std::io::{self, BufReader};
use std::ops::Deref;
use std::str;
use std::vec::Vec;

use crate::bicgr;
use crate::utils::FastaRecord;
use anyhow::Result;
use noodles::fasta;
use rayon::iter::IntoParallelIterator;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

/// Integer Chaos Game Representation (ICGR) for a sequence
/// Using Strings for serialization compatibility.
/// Not all platforms supports 128 bits integers along
/// with Rust versions before 1.26
/// Moreover, serde_json does not yet support i128
/// https://github.com/serde-rs/json/issues/846
/// So the integers will be stored as String and converted as needed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriIntegers {
    /// First integer of the ICGR
    x: String,

    /// Second integer of the ICGR
    y: String,

    /// Block length
    n: usize,
}

/// FromParallelIterator trait implementation for TriIntegersList
impl FromParallelIterator<TriIntegers> for TriIntegersList {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = TriIntegers>,
    {
        let vec: Vec<TriIntegers> = Vec::from_par_iter(par_iter);
        TriIntegersList(vec)
    }
}

impl fmt::Display for TriIntegers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{},{}]", self.x, self.y, self.n)
    }
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct TriIntegersList(Vec<TriIntegers>);

impl<'de> Deserialize<'de> for TriIntegersList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TriIntegersListVisitor;

        impl<'de> Visitor<'de> for TriIntegersListVisitor {
            type Value = TriIntegersList;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a semicolon-separated string of 'x,y,n' triplets")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut result = Vec::new();

                for entry in value.split(';').filter(|s| !s.trim().is_empty()) {
                    let parts: Vec<&str> = entry.split(',').collect();

                    if parts.len() != 3 {
                        return Err(de::Error::custom(format!(
                            "Invalid triplet: '{}'. Expected format 'x,y,n'",
                            entry
                        )));
                    }

                    let x = parts[0].trim().to_string();
                    let y = parts[1].trim().to_string();
                    let n = parts[2].trim().parse::<usize>().map_err(|_| {
                        de::Error::custom(format!("Invalid block length: '{}'", parts[2]))
                    })?;

                    result.push(TriIntegers { x, y, n });
                }

                Ok(TriIntegersList(result))
            }
        }

        deserializer.deserialize_str(TriIntegersListVisitor)
    }
}

impl TriIntegersList {
    pub fn iter(&self) -> std::slice::Iter<'_, TriIntegers> {
        self.0.iter()
    }

    pub fn to_dna(&self, overlap: u8) -> Result<Vec<u8>, String> {
        let dna_chunks: Vec<Vec<u8>> = self
            .iter()
            .map(|x| tri_integers_to_dna(x.clone()))
            .collect();
        let chunks: Vec<&[u8]> = dna_chunks.iter().map(|v| v.as_slice()).collect();
        // merge strings with overlaps
        merge_with_overlap(chunks, overlap as usize)
    }
}

fn merge_with_overlap(chunks: Vec<&[u8]>, overlap: usize) -> Result<Vec<u8>, String> {
    if chunks.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = chunks[0].to_vec();
    for window in chunks.windows(2) {
        let prev = window[0];
        let curr = window[1];

        if prev.len() < overlap || curr.len() < overlap {
            return Err("Chunk too short to contain required overlap".into());
        }

        let prev_tail = &prev[prev.len() - overlap..];
        let curr_head = &curr[..overlap];

        if prev_tail != curr_head {
            return Err(format!(
                "Overlap mismatch: expected {:?}, got {:?}",
                String::from_utf8_lossy(prev_tail),
                String::from_utf8_lossy(curr_head)
            ));
        }
        // Append only the non-overlapping part
        result.extend_from_slice(&curr[overlap..]);
    }
    Ok(result)
}

impl Deref for TriIntegersList {
    type Target = Vec<TriIntegers>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for TriIntegersList {
    type Item = TriIntegers;
    type IntoIter = std::vec::IntoIter<TriIntegers>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for TriIntegersList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self
            .0
            .iter()
            .map(|x| format!("{},{},{}", x.x, x.y, x.n))
            .collect::<Vec<_>>()
            .join(";");
        write!(f, "{}", s)
    }
}

impl<'a> IntoIterator for &'a TriIntegersList {
    type Item = &'a TriIntegers;
    type IntoIter = std::slice::Iter<'a, TriIntegers>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut TriIntegersList {
    type Item = &'a mut TriIntegers;
    type IntoIter = std::slice::IterMut<'a, TriIntegers>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl TriIntegers {
    pub(crate) fn from_sequence(sequence: &[u8], block_length: usize) -> TriIntegersList {
        let seq = String::from_utf8_lossy(sequence);
        let seq_length = seq.len();

        let chunks: Vec<&str> = if seq_length > block_length {
            str_chunks(&seq, block_length).collect()
        } else {
            vec![&seq]
        };

        // Parallelize for large datasets
        chunks.into_par_iter().map(Self::icgr_from_chunk).collect()
    }

    fn icgr_from_chunk(chunk: &str) -> TriIntegers {
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
        TriIntegers {
            x: xx[n - 1].to_string(),
            y: yy[n - 1].to_string(),
            n,
        }
    }
}

/// Decodes ICGR values back into a nucleotide sequence.
fn tri_integers_to_dna(tri_integers: TriIntegers) -> Vec<u8> {
    let mut complete_dna = Vec::with_capacity(tri_integers.n);
    let base: i128 = 2;

    let mut an = vec![0; tri_integers.n];
    let mut bn = vec![0; tri_integers.n];
    an[tri_integers.n - 1] = tri_integers.x.parse().unwrap_or(0);
    bn[tri_integers.n - 1] = tri_integers.y.parse().unwrap_or(0);

    let mut seq = Vec::with_capacity(tri_integers.n);

    for index in (0..tri_integers.n).rev() {
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
    complete_dna
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
pub struct Icgr {
    /// A DNA sequence ID: all characters before first whitespace in sequence header
    pub(crate) id: String,

    /// A DNA sequence description: all characters after first whitespace
    pub(crate) desc: Option<String>,

    /// A vector of ICGR which represent the whole DNA sequence
    pub(crate) tri_integers: TriIntegersList,
}

impl Icgr {
    fn to_bicgr(&self, overlap: u8) -> bicgr::Record {
        bicgr::Record {
            seq_id: self.id.clone(),
            desc: self.desc.clone(),
            overlap,
            tri_integers: self.tri_integers.clone(),
        }
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

pub fn encode<W: io::Write>(
    source: String,
    mut destination: W,
    block_length: usize,
    overlap: u8,
) -> Result<()> {
    let mut reader = fasta::Reader::new(BufReader::new(source.as_bytes()));
    for result in reader.records() {
        let record = result?;
        let icgr = record.to_icgr(block_length);
        let bicgr_format = icgr.to_bicgr(overlap);

        // Also write to destination file if provided
        bicgr_format.write_all(&mut destination)?;
    }

    Ok(())
}

pub fn decode<R: BufRead, W: io::Write>(source: R, mut destination: W) -> Result<()> {
    let records = bicgr::read_from(source)?;

    for record in records {
        destination.write_all(
            format!(
                ">{} {}\n{:?}",
                record.seq_id,
                record.desc.unwrap_or_default(),
                record.tri_integers.to_dna(record.overlap)
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}

/*
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

    /*/
    #[test]
    fn test_from_record() {
        let seq = fasta::Record::new(
            fasta::record::Definition::new("sq0", None),
            fasta::record::Sequence::from(b"ATTGCCGTAA".to_vec()),
        );

        assert_eq!(
            from_record(seq, 10),
            ICGR {
                id: "sq0".to_string(),
                desc: None,
                tri_integers: vec![TriIntegers {
                    x: "515".to_string(),
                    y: "783".to_string(),
                    n: 10
                }]
            }
        );
    }*/

    #[test]
    fn test_ichaos() {
        let ichaos = ICGR {
            id: "sq0".to_string(),
            desc: Some(String::from("")),
            tri_integers: vec![TriIntegers {
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
        let ichaos = ICGR {
            id: "sq0".to_string(),
            desc: Some(String::from("")),
            tri_integers: vec![TriIntegers {
                x: "515".to_string(),
                y: "783".to_string(),
                n: 10,
            }],
        };

        assert_eq!(
            TriIntegers::from_sequence(b"ATTGCCGTAA", 10),
            ichaos.tri_integers
        );
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
*/
