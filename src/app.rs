// Copyright 2021-2025 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use clap::{value_parser, Arg, ArgAction, Command};
use std::path::{Path, PathBuf};

pub fn build_app() -> Command {
    Command::new("fastchaos")
        .override_usage("fastchaos [command] [options] <INFILE>")
        .version("1.0")
        .author("Anicet Ebou <anicet.ebou@gmail.com>")
        .about(
            "Rapid encoding, decoding and analysis of DNA sequences with Chaos Game Representation",
        )
        .subcommand(
            Command::new("encode")
                .about("Encode a DNA sequence into integer Chaos Game Representation")
                .override_usage("fastchaos encode [OPTIONS] [FILE]")
                .version("1.0")
                .author("Anicet Ebou <anicet.ebou@gmail.com>")
                .arg(
                    Arg::new("FILE")
                        .help("sequence in fasta format")
                        .value_name("FILE")
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .help("output file")
                        .short('o')
                        .value_name("FILE")
                        .value_parser(is_existing),
                ),
        )
        .subcommand(
            Command::new("decode")
                .about("Decode a sequence integer Chaos Game Representation to DNA")
                .override_usage("fastchaos decode [options] <INFILE>")
                .version("1.0")
                .author("Anicet Ebou <anicet.ebou@gmail.com>")
                .arg(
                    Arg::new("FILE")
                        .index(1)
                        .help("sequences file in fasta format"),
                )
                .arg(
                    Arg::new("output")
                        .help("decode file to FILE")
                        .short('o')
                        .value_name("FILE"),
                )
                .arg(
                    Arg::new("force")
                        .help("force overwriting output file")
                        .long("force")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("draw")
                .about("Draw Chaos Game Representation to from sequence file")
                .override_usage("fastchaos draw [options] <INFILE>")
                .version("1.0")
                .author("Anicet Ebou <anicet.ebou@gmail.com>")
                .arg(
                    Arg::new("INFILE")
                        .help("sequences file in fasta format")
                        .index(1)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("output")
                        .long("out")
                        .short('o')
                        .value_name("DIR")
                        .help("output images to DIR"),
                ),
        )
        .subcommand(
            Command::new("compare")
                .about("SSIM comparison of chaos game representation images of genomes")
                .override_usage("fastchaos compare [options] <QUERY> <REFERENCE>")
                .version("1.0")
                .author("Anicet Ebou <anicet.ebou@gmail.com>")
                .arg(Arg::new("QUERY").help("query sequence").index(1))
                .arg(Arg::new("REFERENCE").help("reference sequence").index(2))
                .arg(
                    Arg::new("queries")
                        .help("query sequence list")
                        .long("ql")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf))
                        .conflicts_with("QUERY"),
                )
                .arg(
                    Arg::new("refs")
                        .help("reference sequence list")
                        .long("rl")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf))
                        .conflicts_with("REFERENCE"),
                )
                .arg(
                    Arg::new("output")
                        .long("out")
                        .short('o')
                        .value_name("FILE")
                        .help("output result to FILE"),
                )
                .arg(
                    Arg::new("allvsall")
                        .help("all vs all comparison")
                        .long("all")
                        .short('a')
                        .action(ArgAction::SetTrue),
                ),
        )
        .arg(
            Arg::new("threads")
                .long("threads")
                .short('t')
                .value_name("INT")
                .help("number of threads")
                .default_value("1"),
        )
}

fn is_existing(s: &str) -> Result<String, String> {
    if !Path::new(s).exists() {
        Ok(s.to_string())
    } else {
        Err("file should not already exists".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cmd() {
        build_app().debug_assert();
    }
}
