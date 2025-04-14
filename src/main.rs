// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use crate::cli::{Cli, Commands};
use anyhow::{bail, Context};
use clap::Parser;
use itertools::Itertools;
use noodles::fasta;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Read, Write};
use std::path::Path;
mod cgr;
mod cli;
mod icgr;
mod utils;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build_global()?;

    match cli.command {
        Commands::Encode(args) => {
            let mut input = String::new();
            let from_stdin = args.file.as_ref().is_none_or(|p| p == Path::new("-"));
            if from_stdin {
                // Read from stdin
                let stdin = io::stdin();
                let mut stdin_lock = stdin.lock();
                let bytes_read = stdin_lock.read_to_string(&mut input)?;

                if bytes_read == 0 || input.trim().is_empty() {
                    bail!("Error: No input provided via FILE or stdin.\nUse --help for usage.");
                }
            } else {
                let opened_file = File::open(args.file.expect("File argument should be supplied"))?;
                let mut reader = fasta::Reader::new(BufReader::new(opened_file));

                for result in reader.records() {
                    let record = result?;
                    input.push_str(&format!(
                        ">{}\n{}\n",
                        record.name(),
                        String::from_utf8_lossy(record.sequence().as_ref())
                    ));
                }

                if input.trim().is_empty() {
                    bail!("Error: Provided file is empty.");
                }
            }

            let destination: Box<dyn Write> = if let Some(out) = args.output {
                Box::new(File::create(out)?)
            } else {
                Box::new(io::stdout().lock())
            };

            let block_length: usize = args.block_width;

            icgr::encode(input, destination, block_length)?;
        }
        Commands::Decode(args) => {
            let mut input = String::new();
            let from_stdin = args.file.as_ref().is_none_or(|p| p == Path::new("-"));
            if from_stdin {
                // Read from stdin
                let stdin = io::stdin();
                let mut stdin_lock = stdin.lock();
                let bytes_read = stdin_lock.read_to_string(&mut input)?;

                if bytes_read == 0 || input.trim().is_empty() {
                    bail!("Error: No input provided via FILE or stdin.\nUse --help for usage.");
                }
            } else {
                let opened_file = File::open(args.file.expect("File argument should be supplied"))?;
                let mut reader = fasta::Reader::new(BufReader::new(opened_file));

                for result in reader.records() {
                    let record = result?;
                    input.push_str(&format!(
                        ">{}\n{}\n",
                        record.name(),
                        String::from_utf8_lossy(record.sequence().as_ref())
                    ));
                }

                if input.trim().is_empty() {
                    bail!("Error: Provided file is empty.");
                }
            }

            let destination: Box<dyn Write> = if let Some(out) = args.output {
                Box::new(File::create(out)?)
            } else {
                Box::new(io::stdout().lock())
            };

            icgr::decode(input, destination)?;
        }
        Commands::Draw(args) => {
            let source = File::open(args.file)?;
            let out = args.output.unwrap_or(
                std::env::current_dir().expect("Currrent working directory should be valid"),
            );
            cgr::draw(source, &out)?;
        }
        Commands::Compare(args) => {
            let mut qfiles = Vec::new();
            let mut rfiles = Vec::new();

            match (&args.query, &args.reference) {
                (Some(q), Some(r)) => {
                    qfiles.push(q.to_string_lossy().into_owned());
                    rfiles.push(r.to_string_lossy().into_owned());
                }
                (Some(q), None) => {
                    qfiles.push(q.to_string_lossy().into_owned());

                    let ref_file = args.refs.as_ref().context("Missing --refs file")?;
                    rfiles = utils::read_lines(ref_file)?.map_while(Result::ok).collect();
                }
                (None, Some(r)) => {
                    let query_file = args.queries.as_ref().context("Missing --queries file")?;
                    qfiles = utils::read_lines(query_file)?
                        .map_while(Result::ok)
                        .collect();

                    rfiles.push(r.to_string_lossy().into_owned());
                }
                (None, None) => {
                    let query_file = args.queries.as_ref().context("Missing --queries file")?;
                    let ref_file = args.refs.as_ref().context("Missing --refs file")?;

                    qfiles = utils::read_lines(query_file)?
                        .map_while(Result::ok)
                        .collect();
                    rfiles = utils::read_lines(ref_file)?.map_while(Result::ok).collect();
                }
            }

            let mut ssim = Vec::new();

            if args.allvsall {
                qfiles.extend(rfiles.clone());
                for pair in qfiles.into_iter().combinations_with_replacement(2) {
                    ssim.push(cgr::compare_genomes(&pair[0], &pair[1])?);
                }
            } else {
                for (q, r) in qfiles.iter().cartesian_product(&rfiles) {
                    ssim.push(cgr::compare_genomes(q, r)?);
                }
            }

            if let Some(output) = args.output {
                let mut out = OpenOptions::new().append(true).create(true).open(output)?;
                for result in ssim {
                    writeln!(out, "{}", result)?;
                }
            } else {
                for result in ssim {
                    println!("{}", result);
                }
            }
        }
    }

    Ok(())
}
