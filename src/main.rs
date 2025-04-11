// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use itertools::Itertools;
use std::path::Path;
use std::{env, fs, io, io::Write, path::PathBuf};

mod app;
mod cgr;
mod icgr;
mod utils;

fn main() -> anyhow::Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());

    let num_threads = matches
        .get_one::<String>("threads")
        .unwrap_or(&"1".to_owned())
        .parse::<usize>()?;
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()?;

    match matches.subcommand() {
        Some(("encode", m)) => handle_encode(m),
        Some(("decode", m)) => handle_decode(m),
        Some(("draw", m)) => handle_draw(m),
        Some(("compare", m)) => handle_compare(m),
        _ => Ok(()), // No valid subcommand
    }?;

    std::process::exit(0)
}

fn handle_encode(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let input = utils::read_input_fasta_or_stdin(matches).unwrap();

    let destination: Box<dyn Write> = if let Some(output) = matches.get_one::<String>("output") {
        Box::new(
            fs::File::create(output).unwrap_or_else(|_| panic!("Failed to opend file: {}", output)),
        )
    } else {
        Box::new(io::stdout().lock())
    };

    icgr::encode(input, destination)?;

    Ok(())
}

fn handle_decode(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let input = utils::read_input_fasta_or_stdin(matches).unwrap();

    let destination: Box<dyn Write> = if let Some(output) = matches.get_one::<String>("output") {
        if Path::new(output).exists() && !matches.get_flag("force") {
            eprintln!("Error: cannot decode to {}, file already exists", output);
            std::process::exit(1);
        }
        Box::new(
            fs::File::create(output).unwrap_or_else(|_| panic!("Failed to opend file: {}", output)),
        )
    } else {
        Box::new(io::stdout().lock())
    };

    icgr::decode(input, destination)?;

    Ok(())
}

fn handle_draw(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let input = matches.get_one::<PathBuf>("INFILE").unwrap();
    let destination = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let source = fs::File::open(input)?;
    cgr::draw(source, &destination)?;
    Ok(())
}

fn handle_compare(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let mut qfiles = Vec::new();
    let mut rfiles = Vec::new();

    if matches.contains_id("QUERY") && matches.contains_id("REFERENCE") {
        qfiles.push(matches.get_one::<String>("QUERY").unwrap().to_string());
        rfiles.push(matches.get_one::<String>("REFERENCE").unwrap().to_string());
    } else {
        if let Some(q_path) = matches.get_one::<PathBuf>("queries") {
            qfiles = utils::read_lines(q_path)?.map_while(Result::ok).collect();
        }
        if let Some(r_path) = matches.get_one::<PathBuf>("refs") {
            rfiles = utils::read_lines(r_path)?.map_while(Result::ok).collect();
        }
        if matches.contains_id("QUERY") {
            qfiles.push(matches.get_one::<String>("QUERY").unwrap().to_string());
        }
        if matches.contains_id("REFERENCE") {
            rfiles.push(matches.get_one::<String>("REFERENCE").unwrap().to_string());
        }
    }

    let mut ssim = Vec::new();

    if matches.get_flag("allvsall") {
        qfiles.extend(rfiles.clone());
        for pair in qfiles.into_iter().combinations_with_replacement(2) {
            ssim.push(cgr::compare_genomes(&pair[0], &pair[1])?);
        }
    } else {
        for (q, r) in qfiles.iter().cartesian_product(&rfiles) {
            ssim.push(cgr::compare_genomes(q, r)?);
        }
    }

    if let Some(output) = matches.get_one::<PathBuf>("output") {
        let mut out = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output)?;
        for result in ssim {
            writeln!(out, "{}", result)?;
        }
    } else {
        for result in ssim {
            println!("{}", result);
        }
    }

    Ok(())
}
