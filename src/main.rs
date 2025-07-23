// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use crate::cli::{Cli, Commands};
use crate::icgr::{ChaosDecoder, ChaosEncoder};
use anyhow::Context;
use clap::Parser;
use itertools::Itertools;
use noodles::fasta;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

mod bicgr;
mod cgr;
mod cli;
mod error;
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
            let from_stdin = args.file.as_ref().is_none_or(|p| p == Path::new("-"));

            let reader: Box<dyn BufRead> = if from_stdin {
                let stdin = io::stdin();
                let stdin_lock = stdin.lock();
                Box::new(stdin_lock)
            } else {
                let file = File::open(args.file.expect("File argument should be supplied"))?;
                Box::new(BufReader::new(file))
            };

            let mut fasta_reader = fasta::Reader::new(reader);

            let mut destination: Box<dyn Write> = if let Some(out) = args.output {
                Box::new(File::create(out)?)
            } else {
                Box::new(io::stdout().lock())
            };

            let block_length: usize = args.block_width;
            let overlap: u8 = args.overlap;
            let strict: bool = args.strict;

            for result in fasta_reader.records() {
                let record = result?;
                let seq = record.sequence();

                // If the sequence is empty, skip it
                if seq.is_empty() {
                    continue;
                }

                let encoded = seq.as_ref().encode(block_length, overlap, strict)?;
                let bicgr = bicgr::Record {
                    seq_id: record.definition().name().to_string(),
                    desc: record
                        .definition()
                        .description()
                        .map(|desc| desc.to_string()),
                    overlap,
                    tri_integers: encoded,
                };
                bicgr.write_all(&mut destination)?;
            }
        }
        Commands::Decode(args) => {
            let from_stdin = args.file.as_ref().is_none_or(|p| p == Path::new("-"));

            let reader: Box<dyn BufRead> = if from_stdin {
                let stdin = io::stdin();
                let stdin_lock = stdin.lock();
                Box::new(stdin_lock)
            } else {
                let file = File::open(args.file.expect("File argument should be supplied"))?;
                Box::new(BufReader::new(file))
            };

            let mut destination: Box<dyn Write> = if let Some(out) = args.output {
                Box::new(File::create(out)?)
            } else {
                Box::new(io::stdout().lock())
            };

            let records = bicgr::read_from(reader)
                .map_err(|e| format!("Failed to read records: {e}"))
                .unwrap();

            for record in records {
                let seq = record.tri_integers.decode(record.overlap)?;

                writeln!(
                    destination,
                    ">{} {}",
                    record.seq_id,
                    record.desc.unwrap_or_default()
                )?;
                writeln!(destination, "{seq}")?;
            }
        }
        Commands::Draw(args) => {
            let source = File::open(args.file)?;
            cgr::draw(source, args.output)?
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
                    writeln!(out, "{result}")?;
                }
            } else {
                for result in ssim {
                    println!("{result}");
                }
            }
        }
    }

    Ok(())
}
