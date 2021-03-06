// Copyright 2021-2022 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate clap;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{self, PathBuf};

use anyhow::{anyhow, Context, Result};

mod app;
mod cgr;
mod icgr;
mod utils;

fn main() -> Result<()> {
    // Get command line arguments ---------------------------------------------
    let matches = app::build_app().get_matches_from(env::args_os());

    let num_threads: usize = matches
        .value_of_t("threads")
        .unwrap_or_else(|error| error.exit());

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()?;

    // Encode subcommand ------------------------------------------------------
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

    // Decode subcommand ------------------------------------------------------
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

    // Draw subcommand --------------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("draw") {
        match matches.value_of("INFILE") {
            Some(input) => match matches.value_of("output") {
                Some(output) => {
                    let destination = PathBuf::from(output);

                    if input == "-" {
                        cgr::draw(io::stdin(), destination)?;
                    } else {
                        let source = fs::File::open(input)?;
                        cgr::draw(source, destination)?;
                    }
                }

                None => {
                    if input == "-" {
                        cgr::draw(io::stdin(), PathBuf::from("."))?;
                    } else {
                        let source = fs::File::open(input)?;
                        cgr::draw(source, PathBuf::from("."))?;
                    }
                }
            },

            None => match matches.value_of("output") {
                Some(output) => {
                    let destination = PathBuf::from(output);

                    cgr::draw(io::stdin(), destination)?;
                }

                None => {
                    cgr::draw(io::stdin(), PathBuf::from("."))?;
                }
            },
        }

    // Compare subcommand -----------------------------------------------------
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
            match matches.value_of("output") {
                Some(filename) => {
                    let mut file = fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(filename)
                        .expect("Cannot open file");

                    let result = cgr::compare_images(files);

                    for data in result {
                        file.write_all(
                            format!("{}\t{}\t{:.8}\n", data.0, data.1, data.2)
                                .as_bytes(),
                        )
                        .expect("Cannot write to file");
                    }
                }
                None => {
                    let result = cgr::compare_images(files);

                    for data in result {
                        writeln!(
                            io::stdout(),
                            "{}\t{}\t{:.8}",
                            data.0,
                            data.1,
                            data.2,
                        )
                        .expect("Cannot write to file");
                    }
                }
            }
        } else if files.iter().map(|x| x.as_str()).all(|file| {
            path::Path::new(file).extension().unwrap() == "fa"
                || path::Path::new(file).extension().unwrap() == "fas"
                || path::Path::new(file).extension().unwrap() == "fasta"
        }) {
            for fi in files {
                let source = fs::File::open(fi)?;
                cgr::draw(source, PathBuf::from("temp"))?;
            }

            let imgs = fs::read_dir("temp")?
                .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
                .collect::<Result<Vec<_>, io::Error>>()?;

            match matches.value_of("output") {
                Some(filename) => {
                    let mut file = fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(filename)
                        .expect("Cannot open file");

                    let result = cgr::compare_images(imgs);

                    for data in result {
                        file.write_all(
                            format!("{}\t{}\t{:.8}\n", data.0, data.1, data.2)
                                .as_bytes(),
                        )
                        .expect("Cannot write to file");
                    }
                }
                None => {
                    let result = cgr::compare_images(imgs);

                    for data in result {
                        writeln!(
                            io::stdout(),
                            "{}\t{}\t{:.8}",
                            data.0,
                            data.1,
                            data.2,
                        )
                        .expect("Cannot write to file");
                    }
                }
            }
        } else {
            writeln!(
                io::stderr(),
                "Supplied files are not images nor sequences"
            )?;
            std::process::exit(exitcode::DATAERR);
        }
    }

    std::process::exit(exitcode::OK)
}
