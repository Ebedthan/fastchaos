// Copyright 2021-2022 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

use clap::{crate_version, AppSettings, Arg, ColorChoice, Command};

pub fn build_app() -> Command<'static> {
    let clap_color_setting = if std::env::var_os("NO_COLOR").is_none() {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    };

    let app = Command::new("fastchaos")
                .override_usage("fastchaos [command] [options] <INFILE>")
                .version(crate_version!())
                .author("Anicet Ebou <anicet.ebou@gmail.com>")
                .about("Rapid encoding, decoding and analysis of DNA sequences with Chaos Game Representation")
                .color(clap_color_setting)
                .setting(AppSettings::DeriveDisplayOrder)
                .subcommand(Command::new("encode")
                    .about("Encode a DNA sequence into integer Chaos Game Representation")
                    .override_usage("fastchaos encode [options] <INFILE>")
                    .version(crate_version!())
                    .author("Anicet Ebou <anicet.ebou@gmail.com>")
                    .arg(Arg::new("INFILE")
                        .help("sequences file in fasta format")
                        .index(1))
                    .arg(Arg::new("output")
                        .long("out")
                        .short('o')
                        .value_name("FILE")
                        .help("output encoded sequences to FILE")
                        .takes_value(true))
                )
                .subcommand(Command::new("decode")
                    .about("Decode a sequence integer Chaos Game Representation to DNA")
                    .override_usage("fastchaos decode [options] <INFILE>")
                    .version(crate_version!())
                    .author("Anicet Ebou <anicet.ebou@gmail.com>")
                    .arg(Arg::new("INFILE")
                        .help("sequences file in fasta format"))
                    .arg(Arg::new("output")
                        .long("out")
                        .short('o')
                        .value_name("FILE")
                        .help("output decoded sequences to FILE")
                        .takes_value(true))
                )
                .subcommand(Command::new("draw")
                    .about("Draw Chaos Game Representation to from sequence file")
                    .override_usage("fastchaos draw [options] <INFILE>")
                    .version(crate_version!())
                    .author("Anicet Ebou <anicet.ebou@gmail.com>")
                    .arg(Arg::new("INFILE")
                        .help("sequences file in fasta format")
                        .index(1))
                    .arg(Arg::new("output")
                        .long("out")
                        .short('o')
                        .value_name("DIR")
                        .help("output images to DIR")
                        .takes_value(true))
                )
                .subcommand(Command::new("compare")
                    .about("SSIM comparison of chaos game representation images")
                    .override_usage("fastchaos compare [options] <INDIR>")
                    .version(crate_version!())
                    .author("Anicet Ebou <anicet.ebou@gmail.com>")
                    .arg(Arg::new("INDIR")
                        .help("folder of CGR images")
                        .required(true)
                        .index(1))
                    .arg(Arg::new("output")
                        .long("out")
                        .short('o')
                        .value_name("FILE")
                        .help("output result to FILE")
                        .takes_value(true))
                )
                .arg(Arg::new("threads")
                    .long("threads")
                    .short('t')
                    .value_name("INT")
                    .help("number of threads")
                    .default_value("1")
                    .takes_value(true));

    app
}
