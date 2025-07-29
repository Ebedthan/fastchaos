// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use clap::{Args, Parser, Subcommand};
use std::{ffi::OsStr, path::PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "chaoscoder",
    version = "1.0.0",
    author = "Anicet Ebou <anicet.ebou@gmail.com>",
    about = "Rapid encoding, decoding and analysis of DNA sequences with (Integer) Chaos Game Representation"
)]
pub struct Cli {
    /// Number of threads to use
    #[arg(short, long, default_value_t = 1)]
    pub threads: usize,

    /// Force overwriting output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub force: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Encode a DNA sequence into Integer Chaos Game Representation
    Encode(EncodeArgs),

    /// Decode a sequence Integer Chaos Game Representation to DNA
    Decode(DecodeArgs),

    /// Draw Chaos Game Representation form sequence file
    Draw(DrawArgs),

    /// Structural Similarity Index Measure (SSIM) comparison of Chaos Game Representation images of genomes
    Compare(CompareArgs),
}

#[derive(Args, Debug)]
pub struct EncodeArgs {
    /// Input sequence file in FASTA format (use '-' for stdin)
    pub file: Option<PathBuf>,

    /// Output file
    #[arg(short, value_parser = must_not_exist)]
    pub output: Option<PathBuf>,

    /// Sequence block length
    #[arg(short = 'w', default_value_t = 100, value_name = "INT", value_parser = validate_block_width)]
    pub block_width: usize,

    /// Sequence overlap
    #[arg(long = "ovl", default_value_t = 5, value_name = "INT", value_parser = validate_overlap)]
    pub overlap: u8,

    /// Strict mode that errors out if unknown characters are found
    #[arg(long = "strict", action = clap::ArgAction::SetTrue)]
    pub strict: bool,
}

#[derive(Args, Debug)]
pub struct DecodeArgs {
    /// Input file to decode
    pub file: Option<PathBuf>,

    /// Output file
    #[arg(short)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct DrawArgs {
    /// Input sequence file in FASTA format
    pub file: PathBuf,

    /// Output file name (.png), defaults to sequence ID
    #[arg(short, value_parser = validate_image_output)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct CompareArgs {
    /// Query sequence file
    pub query: Option<PathBuf>,

    /// Reference sequence file
    pub reference: Option<PathBuf>,

    /// File containing list of query sequences
    #[arg(long, conflicts_with = "query")]
    pub queries: Option<PathBuf>,

    /// File containing list of reference sequences
    #[arg(long, conflicts_with = "reference")]
    pub refs: Option<PathBuf>,

    /// Output result to file
    #[arg(short)]
    pub output: Option<PathBuf>,

    /// Enable all-vs-all comparison
    #[arg(short = 'a', action = clap::ArgAction::SetTrue)]
    pub allvsall: bool,
}

fn must_not_exist(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        Err(format!("{} should not already exist.", path.display()))
    } else {
        Ok(path)
    }
}

fn validate_image_output(s: &str) -> Result<PathBuf, String> {
    let mp = PathBuf::from(s);

    if mp.exists() {
        return Err(format!("{} should not already exist.", mp.display()));
    }
    if let Some(ext) = mp.extension() {
        if ext != OsStr::new("png") {
            return Err(format!(
                "{} should have png extension (.png).",
                mp.display()
            ));
        }
    } else {
        return Err("Output file must have png extension (.png)".to_string());
    }

    Ok(mp)
}

fn validate_block_width(val: &str) -> Result<usize, String> {
    match val.parse::<usize>() {
        Ok(v) => {
            if v <= 100 {
                Ok(v)
            } else {
                Err(String::from("block_widht must be less or equal to 100"))
            }
        }
        Err(_) => Err(String::from("block_width must be a number")),
    }
}

fn validate_overlap(val: &str) -> Result<u8, String> {
    match val.parse::<u8>() {
        Ok(v) => {
            if v > 0 && v <= 20 {
                Ok(v)
            } else {
                Err(String::from("overlap must be between 1 and 20"))
            }
        }
        Err(_) => Err(String::from("overlap must be a number")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_must_not_exist_with_nonexistent_file() {
        let path = "target/tmp_nonexistent_file.txt";
        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path).unwrap();
        }
        let result = must_not_exist(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::path::PathBuf::from(path));
    }

    #[test]
    fn test_must_not_exist_with_existing_file() {
        let path = "target/tmp_existing_file.txt";
        File::create(path).unwrap();
        let result = must_not_exist(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("should not already exist"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_validate_image_output_with_valid_png_path() {
        let path = "target/tmp_image.png";
        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path).unwrap();
        }
        let result = validate_image_output(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::path::PathBuf::from(path));
    }

    #[test]
    fn test_validate_image_output_with_wrong_extension() {
        let result = validate_image_output("target/image.jpg");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("should have png extension"));
    }

    #[test]
    fn test_validate_image_output_with_missing_extension() {
        let result = validate_image_output("target/image");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Output file must have png extension (.png)"
        );
    }

    #[test]
    fn test_validate_image_output_with_existing_file() {
        let path = "target/existing.png";
        File::create(path).unwrap();
        let result = validate_image_output(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("should not already exist"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_validate_block_width_valid() {
        let result = validate_block_width("50");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
    }

    #[test]
    fn test_validate_block_width_non_number() {
        let result = validate_block_width("abc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "block_width must be a number");
    }

    #[test]
    fn test_validate_block_width_too_large() {
        let result = validate_block_width("101");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "block_widht must be less or equal to 100"
        );
    }

    #[test]
    fn test_validate_overlap_valid() {
        let result = validate_overlap("10");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_validate_overlap_non_number() {
        let result = validate_overlap("xyz");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "overlap must be a number");
    }

    #[test]
    fn test_validate_overlap_zero() {
        let result = validate_overlap("0");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "overlap must be between 1 and 20");
    }

    #[test]
    fn test_validate_overlap_above_limit() {
        let result = validate_overlap("21");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "overlap must be between 1 and 20");
    }
}
