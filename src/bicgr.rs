// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use crate::icgr::TriIntegersList;
use serde::Deserialize;
use std::io::{self, BufRead, Write};

/// This struct define the block-based integer chaos game representation file format.
/// The BICGR is a tsv-like file format.
#[derive(Debug, Deserialize)]
pub struct Record {
    /// Sequence ID
    pub(crate) seq_id: String,

    /// Sequence description
    pub(crate) desc: Option<String>,

    /// Overlapping sequence length
    pub(crate) overlap: u8,

    /// The tri-integers of ICGR
    pub(crate) tri_integers: TriIntegersList,
}

impl Record {
    pub fn write_all<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let desc = self.desc.clone().unwrap_or_default();
        writeln!(
            writer,
            "{}\t{}\t{}\t{}",
            self.seq_id, desc, self.overlap, self.tri_integers
        )
    }
}

pub fn read_from<R: BufRead>(reader: R) -> io::Result<Vec<Record>> {
    let mut records = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(reader);
    for (i, result) in rdr.deserialize::<Record>().enumerate() {
        match result {
            Ok(record) => {
                if record.seq_id.trim().is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Missing sequence ID at line {}", i + 1),
                    ));
                }
                if record.overlap == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid overlap (0) at line {}", i + 1),
                    ));
                }
                records.push(record);
            }
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Error parsing record at line {}: {}", i + 1, e),
                ));
            }
        }
    }
    Ok(records)
}
