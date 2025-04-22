// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use crate::icgr::TriIntegersList;
use serde::Deserialize;
use std::io::{self, BufRead, Write};

/// Block-based Integer Chaos Game Representation (BICGR) File Format
///
/// This format is used to store encoded DNA sequences in a tab-separated structure.
/// It supports efficient serialization and deserialization of sequences encoded using iCGR.
///
/// ### BNF Grammar of BICGR file:
/// ```text
/// <bicgr_file>    ::= <header_line> <sequence_line>+
/// <header_line>   ::= "#seq_id" "\t" "description" "\t" "overlap" "\t" "tri_integers" "\n"
/// <sequence_line> ::= <seq_id> "\t" <description> "\t" <overlap> "\t" <tri_integers> "\n"
///
/// <seq_id>         ::= [^\t\n]+
/// <description>    ::= [^\t\n]*
/// <overlap>        ::= [0-9]+
/// <tri_integers>   ::= <tri_integer> (";" <tri_integer>)*
/// <tri_integer>    ::= [0-9]+ "," [0-9]+
/// ```
///
/// A single BICGR record representing one encoded sequence.
#[derive(Debug, Deserialize)]
pub struct Record {
    /// Unique sequence identifier.
    pub(crate) seq_id: String,

    /// Optional free-text description of the sequence.
    pub(crate) desc: Option<String>,

    /// The overlap value used when the sequence was divided into blocks.
    pub(crate) overlap: u8,

    /// Encoded sequence data in the form of tri-integers.
    pub(crate) tri_integers: TriIntegersList,
}

impl Record {
    /// Writes a single BICGR record to a writer (e.g. file or stdout).
    ///
    /// Output format is tab-separated and matches the expected input format for deserialization.
    ///
    /// # Example output:
    /// ```text
    /// seq1\tSome description\t8\t1024,2048,30;512,1024,20
    /// ```
    pub fn write_all<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let desc = self.desc.clone().unwrap_or_default();
        writeln!(
            writer,
            "{}\t{}\t{}\t{}",
            self.seq_id, desc, self.overlap, self.tri_integers
        )
    }
}

/// Reads and parses BICGR records from a buffered reader (e.g. file or stdin).
///
/// Expects a tab-separated format with no headers (can skip a comment header manually).
///
/// # Errors
/// Returns an `io::Error` in the following cases:
/// - Missing or empty sequence ID
/// - Overlap value of zero
/// - Invalid formatting or deserialization failure
pub fn read_from<R: BufRead>(reader: R) -> io::Result<Vec<Record>> {
    let mut records = Vec::new();

    // Set up CSV reader for tab-delimited, no-header format.
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(reader);

    // Process each line and deserialize into a Record.
    for (i, result) in rdr.deserialize::<Record>().enumerate() {
        match result {
            Ok(record) => {
                // Validate essential fields
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
            // Report and propagate deserialization errors
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icgr::TriIntegers;
    use std::io::Cursor;

    fn make_input(data: &str) -> Cursor<Vec<u8>> {
        Cursor::new(data.as_bytes().to_vec())
    }

    #[test]
    fn test_read_valid_record() {
        let input = "seq1\tdescription\t8\t1,2,3;4,5,6\n";
        let reader = make_input(input);
        let records = read_from(reader).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].seq_id, "seq1");
        assert_eq!(records[0].desc.as_deref(), Some("description"));
        assert_eq!(records[0].overlap, 8);
        assert_eq!(records[0].tri_integers.to_string(), "1,2,3;4,5,6");
    }

    #[test]
    fn test_read_missing_seq_id() {
        let input = "\tDescription\t8\t1,2,3;3,4,5\n";
        let reader = make_input(input);
        let result = read_from(reader);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Missing sequence ID"));
    }

    #[test]
    fn test_read_invalid_overlap() {
        let input = "seq2\tDesc\t0\t1,2,3;3,4,5\n";
        let reader = make_input(input);
        let result = read_from(reader);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid overlap"));
    }

    #[test]
    fn test_read_malformed_line() {
        let input = "seq3\tOnlyOneField\n";
        let reader = make_input(input);
        let result = read_from(reader);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("parsing record"));
    }

    #[test]
    fn test_write_all() {
        let record = Record {
            seq_id: "seq1".to_string(),
            desc: Some("mydesc".to_string()),
            overlap: 8,
            tri_integers: TriIntegersList::new(vec![
                TriIntegers::new(1, 2, 3),
                TriIntegers::new(4, 5, 6),
            ]),
        };

        let mut output = Vec::new();
        record.write_all(&mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "seq1\tmydesc\t8\t1,2,3;4,5,6\n");
    }

    #[test]
    fn test_write_all_without_description() {
        let record = Record {
            seq_id: "seqX".to_string(),
            desc: None,
            overlap: 10,
            tri_integers: TriIntegersList::new(vec![TriIntegers::new(7, 8, 9)]),
        };

        let mut output = Vec::new();
        record.write_all(&mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "seqX\t\t10\t7,8,9\n");
    }
}
