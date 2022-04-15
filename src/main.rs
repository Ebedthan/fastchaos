// Copyright 2021-2022 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate noodles;
extern crate tempfile;

use std::env;
use std::fs;
use std::io;
use std::path;

use anyhow::{anyhow, Context, Result};
use noodles::fasta;
use tempfile::tempdir;

mod app;
mod cgr;
mod icgr;
mod utils;

fn main() -> Result<()> {
    // Get command line arguments ---------------------------------------------
    let matches = app::build_app().get_matches_from(env::args_os());

    let num_threads: usize = matches
        .value_of("threads")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap();

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()?;

    // Encode mode ------------------------------------------------------------
    if let Some(matches) = matches.subcommand_matches("encode") {
        match matches.value_of("INFILE") {
            Some(input) => match matches.value_of("output") {
                Some(output) => {
                    let destination = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output)?;

                    if input == "-" {
                        icgr::encode(io::stdin(), destination)?;
                    } else {
                        let source = fs::File::open(input)?;
                        icgr::encode(source, destination)?;
                    }
                }

                None => {
                    if input == "-" {
                        icgr::encode(io::stdin(), io::stdout())?;
                    } else {
                        let source = fs::File::open(input)?;
                        icgr::encode(source, io::stdout())?;
                    }
                }
            },

            None => match matches.value_of("output") {
                Some(output) => {
                    let destination = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output)?;

                    icgr::encode(io::stdin(), destination)?;
                }

                None => {
                    icgr::encode(io::stdin(), io::stdout())?;
                }
            },
        }

    // Decode mode ------------------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("decode") {
        match matches.value_of("INFILE") {
            Some(input) => match matches.value_of("output") {
                Some(output) => {
                    let destination = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output)?;

                    if input == "-" {
                        icgr::decode(io::stdin(), destination)?;
                    } else {
                        let source = fs::File::open(input)?;
                        icgr::decode(source, destination)?;
                    }
                }

                None => {
                    if input == "-" {
                        icgr::decode(io::stdin(), io::stdout())?;
                    } else {
                        let source = fs::File::open(input)?;
                        icgr::decode(source, io::stdout())?;
                    }
                }
            },

            None => match matches.value_of("output") {
                Some(output) => {
                    let destination = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output)?;

                    icgr::decode(io::stdin(), destination)?;
                }

                None => {
                    icgr::decode(io::stdin(), io::stdout())?;
                }
            },
        }

    // Draw from sequence file ------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("draw") {
        let dir = tempdir()?;
        let infile = dir.path().join("temporary.fa");
        let ic = infile.clone();
        let cc = ic.to_str();

        let file = match matches.value_of("INFILE") {
            Some(value) => {
                // Read from stdin
                if value == "-" {
                    let mut writer = fasta::Writer::new(
                        fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(infile)?,
                    );
                    let mut reader =
                        fasta::Reader::new(io::BufReader::new(io::stdin()));

                    for result in reader.records() {
                        let record = result?;

                        writer.write_record(&record)?;
                    }

                    cc.unwrap().to_string()

                // A file is specified, read from it...
                } else {
                    value.to_string()
                }
            }

            // Read from stdin
            None => {
                let mut writer = fasta::Writer::new(
                    fs::OpenOptions::new().append(true).open(infile)?,
                );
                let mut reader =
                    fasta::Reader::new(io::BufReader::new(io::stdin()));

                for result in reader.records() {
                    let record = result?;

                    writer.write_record(&record)?;
                }

                cc.unwrap().to_string()
            }
        };

        cgr::draw_from_file(&file, matches.value_of("output"))?;
    } else if let Some(matches) = matches.subcommand_matches("compare") {
        let folder = matches
            .value_of("INDIR")
            .with_context(|| anyhow!("Could not find input directory"))?;

        let files = fs::read_dir(folder)?
            .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        if files
            .iter()
            .map(|x| x.as_str())
            .all(|file| path::Path::new(file).extension().unwrap() == "png")
        {
            cgr::compare_images(files, matches.value_of("output"))?;
        } else if files.iter().map(|x| x.as_str()).all(|file| {
            path::Path::new(file).extension().unwrap() == "fa"
                || path::Path::new(file).extension().unwrap() == "fas"
                || path::Path::new(file).extension().unwrap() == "fasta"
        }) {
            for fi in files {
                cgr::draw_from_file(&fi, Some("temp"))?;
            }

            let imgs = fs::read_dir("temp")?
                .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
                .collect::<Result<Vec<_>, io::Error>>()?;

            cgr::compare_images(imgs, matches.value_of("output"))?;
        } else {
            println!("Supplied files are not images nor sequences");
            std::process::exit(exitcode::DATAERR);
        }
    }

    Ok(())
}
