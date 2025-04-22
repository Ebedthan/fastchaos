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
use crate::error::IcgrError;
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

        impl Visitor<'_> for TriIntegersListVisitor {
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
    pub fn new(tri_integers: Vec<TriIntegers>) -> Self {
        TriIntegersList(tri_integers)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, TriIntegers> {
        self.0.iter()
    }

    pub fn to_dna(&self, overlap: u8) -> Result<String, IcgrError> {
        let dna_chunks: Vec<Vec<u8>> = self
            .iter()
            .map(|x| tri_integers_to_dna(x.clone()))
            .collect();
        let chunks: Vec<&[u8]> = dna_chunks.iter().map(|v| v.as_slice()).collect();
        // merge strings with overlaps
        let merged: Vec<u8> = merge_with_overlap(chunks, overlap as usize)?;

        // Validate total length: sum of all n - overlap * (k - 1)
        let expected_len: usize = self.iter().map(|t| t.n).sum::<usize>()
            - (overlap as usize * (self.len().saturating_sub(1)));
        if merged.len() != expected_len {
            return Err(IcgrError::OverlapMismatch {
                expected: format!("{}", expected_len),
                actual: format!("{}", merged.len()),
            });
        }
        Ok(String::from_utf8(merged)?)
    }
}

fn merge_with_overlap(chunks: Vec<&[u8]>, overlap: usize) -> Result<Vec<u8>, IcgrError> {
    if chunks.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = chunks[0].to_vec();

    for window in chunks.windows(2) {
        let prev = window[0];
        let curr = window[1];

        if prev.len() < overlap || curr.len() < overlap {
            return Err(IcgrError::ChunkTooShort);
        }

        let prev_tail = &prev[prev.len() - overlap..];
        let curr_head = &curr[..overlap];

        if prev_tail != curr_head {
            return Err(IcgrError::OverlapMismatch {
                expected: String::from_utf8_lossy(prev_tail).to_string(),
                actual: String::from_utf8_lossy(curr_head).to_string(),
            });
        }
        // Push only the non-overlapping portion of curr
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
    pub fn new(x: u8, y: u8, n: usize) -> Self {
        TriIntegers {
            x: format!("{x}"),
            y: format!("{y}"),
            n,
        }
    }

    pub(crate) fn from_sequence(
        sequence: &[u8],
        block_length: usize,
        overlap: u8,
        strict: bool,
    ) -> Result<TriIntegersList, IcgrError> {
        let seq = String::from_utf8_lossy(sequence);

        let chunks: Vec<&str> = if seq.len() > block_length {
            str_chunks_overlap(&seq, block_length, overlap as usize).collect()
        } else {
            vec![&seq]
        };

        let icgrs = chunks
            .into_par_iter()
            .map(|chunk| Self::icgr_from_chunk(chunk, strict))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TriIntegersList(icgrs))
    }

    fn icgr_from_chunk(chunk: &str, strict: bool) -> Result<TriIntegers, IcgrError> {
        let base: i128 = 2;
        let mut xx = Vec::with_capacity(chunk.len());
        let mut yy = Vec::with_capacity(chunk.len());

        for (index, nucleotide) in chunk.chars().enumerate() {
            let new_index = index as u32;
            let (aa, bb) = match index {
                0 => match nucleotide {
                    'A' => (1, 1),
                    'T' => (-1, 1),
                    'C' => (-1, -1),
                    'G' => (1, -1),
                    _ if strict => return Err(IcgrError::UnknownNucleotide(nucleotide)),
                    _ => (0, 0),
                },
                _ => {
                    let (prev_x, prev_y) = (xx[index - 1], yy[index - 1]);
                    let power = base.pow(new_index);
                    match nucleotide {
                        'A' => (prev_x + power, prev_y + power),
                        'T' => (prev_x - power, prev_y + power),
                        'C' => (prev_x - power, prev_y - power),
                        'G' => (prev_x + power, prev_y - power),
                        _ if strict => return Err(IcgrError::UnknownNucleotide(nucleotide)),
                        _ => (prev_x, prev_y),
                    }
                }
            };
            xx.push(aa);
            yy.push(bb);
        }

        Ok(TriIntegers {
            x: xx.last().unwrap().to_string(),
            y: yy.last().unwrap().to_string(),
            n: chunk.len(),
        })
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
fn str_chunks_overlap<'a>(
    s: &'a str,
    chunk_size: usize,
    overlap: usize,
) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    assert!(
        chunk_size > overlap,
        "chunk_size must be greater than overlap"
    );
    Box::new(
        (0..s.len())
            .step_by(chunk_size - overlap)
            .take_while(move |&start| start < s.len())
            .map(move |start| {
                let end = usize::min(start + chunk_size, s.len());
                &s[start..end]
            }),
    )
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
        let icgr = record.to_icgr(block_length, overlap);
        let bicgr_format = icgr.to_bicgr(overlap);

        // Also write to destination file if provided
        bicgr_format.write_all(&mut destination)?;
    }

    Ok(())
}

pub fn decode<R: BufRead, W: io::Write>(source: R, mut destination: W) -> Result<(), String> {
    let records = bicgr::read_from(source).map_err(|e| format!("Failed to read records: {}", e))?;

    for record in records {
        let seq = record
            .tri_integers
            .to_dna(record.overlap)
            .map_err(|e| format!("Failed to convert record '{}' to DNA: {}", record.seq_id, e))?;

        writeln!(
            destination,
            ">{} {}",
            record.seq_id,
            record.desc.unwrap_or_default()
        )
        .map_err(|e| format!("Failed to write header: {}", e))?;
        writeln!(destination, "{}", seq).map_err(|e| format!("Failed to write sequence: {}", e))?;
    }

    Ok(())
}

pub trait ChaosEncoder {
    fn encode(&self, block_length: usize, overlap: u8) -> Result<TriIntegersList, IcgrError>;
}

pub trait ChaosDecoder {
    fn decode(&self, overlap: u8) -> Result<String, IcgrError>;
}

impl ChaosEncoder for [u8] {
    fn encode(&self, block_length: usize, overlap: u8) -> Result<TriIntegersList, IcgrError> {
        TriIntegers::from_sequence(self, block_length, overlap, true)
    }
}

impl ChaosDecoder for TriIntegersList {
    fn decode(&self, overlap: u8) -> Result<String, IcgrError> {
        self.to_dna(overlap)
    }
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
