// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "fastchaos",
    version = "1.0",
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

    /// Sequence block length (either 50 or 100)
    #[arg(short = 'w', default_value_t = 100, value_name = "INT", value_parser = validate_block_width)]
    pub block_width: usize,

    /// Sequence overlap
    #[arg(long = "ovl", default_value_t = 10, value_name = "INT", value_parser = validate_overlap)]
    pub overlap: u8,
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

    /// Output directory for images
    #[arg(short)]
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

fn validate_block_width(val: &str) -> Result<usize, String> {
    match val.parse::<usize>() {
        Ok(50) | Ok(100) => Ok(val.parse().unwrap()),
        Ok(_) => Err(String::from("block_width must be 50 or 100")),
        Err(_) => Err(String::from("block_width must be a number")),
    }
}

fn validate_overlap(val: &str) -> Result<u8, String> {
    match val.parse::<u8>() {
        Ok(v) => {
            if v > 0 && v <= 10 {
                Ok(v)
            } else {
                Err(String::from("overlap must be between 1 and 10"))
            }
        }
        Err(_) => Err(String::from("overlap must be a number")),
    }
}
