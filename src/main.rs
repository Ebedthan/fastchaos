// Copyright 2021-2023 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;
extern crate clap;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{self, Path, PathBuf};

use anyhow::{anyhow, Context, Result};

mod app;
mod cgr;
mod icgr;
mod utils;

fn main() -> Result<()> {
    // Get command line arguments ---------------------------------------------
    let matches = app::build_app().get_matches_from(env::args_os());

    let num_threads = matches
        .get_one::<String>("threads")
        .unwrap_or(&"1".to_owned())
        .parse::<usize>()
        .unwrap();

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()?;

    // Encode subcommand ------------------------------------------------------
    if let Some(matches) = matches.subcommand_matches("encode") {
        let input = matches.get_one::<PathBuf>("INFILE").unwrap();
        let output = format!("{}.cgr", input.display());
        let destination = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output)?;

        let source = fs::File::open(input)?;
        icgr::encode(source, destination)?;

    // Decode subcommand ------------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("decode") {
        let input = matches.get_one::<PathBuf>("INFILE").unwrap();
        if input.extension() != Some(std::ffi::OsStr::new("cgr")) {
            eprintln!(
                "error: unknown suffix ({:?}), ignored",
                input.extension().unwrap()
            );
            std::process::exit(1);
        }
        match matches.get_one::<PathBuf>("output") {
            Some(output) => {
                if Path::new(output).exists() {
                    eprintln!(
                        "error: cannot decode to '{:?}', file already exists",
                        output
                    );
                    std::process::exit(1);
                }
                let destination = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output)?;

                let source = fs::File::open(input)?;
                icgr::decode(source, destination)?;
            }
            None => {
                let output = input.as_path().with_extension("cgr");
                if output.exists() {
                    eprintln!(
                        "error: cannot decode to {:?}, file already exists",
                        output
                    );
                    std::process::exit(1);
                }
                let destination = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output)?;

                let source = fs::File::open(input)?;
                icgr::decode(source, destination)?;
            }
        }

        // Delete cgr file if not otherwise specified
        if !matches.get_flag("keep") {
            std::fs::remove_file(input)?;
        }

    // Draw subcommand --------------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("draw") {
        match matches.get_one::<String>("INFILE") {
            Some(input) => match matches.get_one::<String>("output") {
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

            None => match matches.get_one::<String>("output") {
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
            .get_one::<String>("INDIR")
            .with_context(|| anyhow!("Could not find input directory"))?;

        let files = fs::read_dir(folder)?
            .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        if files
            .iter()
            .map(|x| x.as_str())
            .all(|file| path::Path::new(file).extension().unwrap() == "png")
        {
            match matches.get_one::<String>("output") {
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

            match matches.get_one::<String>("output") {
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
