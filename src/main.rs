// Copyright 2021 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.
extern crate anyhow;
extern crate bio;
extern crate exitcode;

use std::env;

use anyhow::{anyhow, Context, Result};

mod app;
mod utils;

fn main() -> Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());

    // Encode sequence
    if let Some(matches) = matches.subcommand_matches("encode") {
        let file = matches
            .value_of("INFILE")
            .with_context(|| anyhow!("Could not find input file"))?;

        utils::encode_from_file(file, matches.value_of("out"))?;
    } else if let Some(matches) = matches.subcommand_matches("decode") {
        let file = matches
            .value_of("INFILE")
            .with_context(|| anyhow!("Could not find input file"))?;

        utils::decode_from_file(file, matches.value_of("out"))?;
    } else {
        println!("Done");
    }

    Ok(())
}
