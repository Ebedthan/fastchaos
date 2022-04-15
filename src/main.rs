// Copyright 2021-2022 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use std::env;
use std::fs;
use std::io;
use std::path;

use anyhow::{anyhow, Context, Result};

mod app;
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
        let file = matches
            .value_of("INFILE")
            .with_context(|| anyhow!("Could not find input file"))?;

        utils::encode_from_file(file, matches.value_of("output"))?;

    // Decode mode ------------------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("decode") {
        let file = matches
            .value_of("INFILE")
            .with_context(|| anyhow!("Could not find input file"))?;

        utils::decode_from_file(file, matches.value_of("output"))?;

    // Draw from sequence file ------------------------------------------------
    } else if let Some(matches) = matches.subcommand_matches("draw") {
        let file = matches
            .value_of("INFILE")
            .with_context(|| anyhow!("Could not find input file"))?;

        utils::draw_from_file(file, matches.value_of("output"))?;
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
            utils::compare_images(files, matches.value_of("output"))?;
        } else if files.iter().map(|x| x.as_str()).all(|file| {
            path::Path::new(file).extension().unwrap() == "fa"
                || path::Path::new(file).extension().unwrap() == "fas"
                || path::Path::new(file).extension().unwrap() == "fasta"
        }) {
            for fi in files {
                utils::draw_from_file(&fi, Some("temp"))?;
            }

            let imgs = fs::read_dir("temp")?
                .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
                .collect::<Result<Vec<_>, io::Error>>()?;

            utils::compare_images(imgs, matches.value_of("output"))?;
        } else {
            println!("Supplied files are not images nor sequences");
            std::process::exit(exitcode::DATAERR);
        }
    }

    Ok(())
}
