// Copyright 2021-2024 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use anyhow::Result;
use itertools::Itertools;

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
        let output = format!("{}.icgr", input.display());
        let destination = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output)?;

        let source = fs::File::open(input)?;
        icgr::encode(source, destination)?;

        if !matches.get_flag("keep") {
            fs::remove_file(input)?;
        }

    // Decode subcommand ------------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("decode") {
        let input = matches.get_one::<PathBuf>("INFILE").unwrap();
        if input.extension() != Some(std::ffi::OsStr::new("icgr")) {
            eprintln!(
                "error: unknown suffix ({:?}), ignored",
                input.extension().unwrap()
            );
            std::process::exit(1);
        }
        match matches.get_one::<PathBuf>("output") {
            Some(output) => {
                if Path::new(output).exists() && !matches.get_flag("force") {
                    eprintln!(
                        "error: cannot decode to '{:?}', file already exists",
                        output
                    );
                    std::process::exit(1);
                } else {
                    let destination = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output)?;

                    let source = fs::File::open(input)?;
                    icgr::decode(source, destination)?;
                }
            }
            None => {
                let output = input.as_path().with_extension("icgr"); // TODO: Verify
                if output.exists() && !matches.get_flag("force") {
                    eprintln!(
                        "error: cannot decode to {:?}, file already exists",
                        output
                    );
                    std::process::exit(1);
                } else {
                    let destination = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output)?;

                    let source = fs::File::open(input)?;
                    icgr::decode(source, destination)?;
                }
            }
        }

        // Delete cgr file if not otherwise specified
        if !matches.get_flag("keep") {
            fs::remove_file(input)?;
        }

    // Draw subcommand --------------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("draw") {
        let input = matches.get_one::<PathBuf>("INFILE").unwrap();
        match matches.get_one::<String>("output") {
            Some(output) => {
                let destination = PathBuf::from(output);
                let source = fs::File::open(input)?;
                let _ = cgr::draw(source, destination)?;
            }

            None => {
                let source = fs::File::open(input)?;
                let _ = cgr::draw(source, PathBuf::from("."))?;
            }
        }

    // Compare subcommand -----------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("compare") {
        let mut ssim = Vec::new();

        if matches.contains_id("QUERY") && matches.contains_id("REFERENCE") {
            ssim.push(cgr::compare_genomes(
                matches.get_one::<String>("QUERY").unwrap(),
                matches.get_one::<String>("REFERENCE").unwrap(),
            )?);
        } else {
            let mut qfiles = Vec::new();
            let mut rfiles = Vec::new();
            if matches.contains_id("QUERY") && matches.contains_id("refs") {
                if let Ok(lines) = utils::read_lines(
                    matches.get_one::<PathBuf>("refs").unwrap(),
                ) {
                    for line in lines.map_while(Result::ok) {
                        rfiles.push(line);
                    }
                }
                qfiles.push(
                    matches.get_one::<String>("QUERY").unwrap().to_string(),
                );
            } else if matches.contains_id("REFERENCE")
                && matches.contains_id("queries")
            {
                if let Ok(lines) = utils::read_lines(
                    matches.get_one::<PathBuf>("queries").unwrap(),
                ) {
                    for line in lines.map_while(Result::ok) {
                        qfiles.push(line);
                    }
                }
                rfiles.push(
                    matches.get_one::<String>("REFERENCE").unwrap().to_string(),
                );
            } else {
                if let Ok(lines) = utils::read_lines(
                    matches.get_one::<PathBuf>("queries").unwrap(),
                ) {
                    for line in lines.map_while(Result::ok) {
                        qfiles.push(line);
                    }
                }
                if let Ok(lines) = utils::read_lines(
                    matches.get_one::<PathBuf>("refs").unwrap(),
                ) {
                    for line in lines.map_while(Result::ok) {
                        rfiles.push(line);
                    }
                }
            }
            if matches.get_flag("allvsall") {
                qfiles.extend(rfiles);

                let it = qfiles.into_iter().combinations_with_replacement(2);

                for comb in it {
                    ssim.push(cgr::compare_genomes(&comb[0], &comb[1])?);
                }
            } else {
                let it = qfiles.iter().cartesian_product(rfiles.iter());

                for prod in it {
                    ssim.push(cgr::compare_genomes(prod.0, prod.1)?);
                }
            }
        }

        if matches.contains_id("output") {
            let mut out = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(matches.get_one::<PathBuf>("output").unwrap())?;

            for result in ssim {
                out.write_all(format!("{}", result).as_bytes())?;
            }
        } else {
            for result in ssim {
                println!("{}", result);
            }
        }
    }

    std::process::exit(0)
}
