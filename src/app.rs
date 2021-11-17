use clap::{crate_version, App, AppSettings, Arg, SubCommand};

pub fn build_app() -> App<'static, 'static> {
    let clap_color_setting = if std::env::var_os("NO_COLOR").is_none() {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };

    let app = App::new("fastchaos")
                .version(crate_version!())
                .author("Anicet Ebou <anicet.ebou@gmail.com>")
                .about("Rapid encoding, decoding and analysis of DNA sequences with Chaos Game Representation")
                .setting(clap_color_setting)
                .setting(AppSettings::DeriveDisplayOrder)
                .subcommand(SubCommand::with_name("encode")
                    .about("Encode a DNA sequence into integer Chaos Game Representation")
                    .version(crate_version!())
                    .author("Anicet Ebou <anicet.ebou@gmail.com>")
                    .arg(Arg::with_name("INFILE")
                        .help("Sets the input file to use")
                        .required(true)
                        .index(1))
                    .arg(Arg::with_name("output")
                        .long("out")
                        .short("o")
                        .value_name("FILE")
                        .help("Sets a file output name")
                        .takes_value(true))
                )
                .subcommand(SubCommand::with_name("decode")
                    .about("Decode a sequence integer Chaos Game Representation to DNA")
                    .version(crate_version!())
                    .author("Anicet Ebou <anicet.ebou@gmail.com>")
                    .arg(Arg::with_name("INFILE")
                        .help("Sets the input file to use")
                        .required(true)
                        .index(1))
                    .arg(Arg::with_name("output")
                        .long("out")
                        .short("o")
                        .value_name("FILE")
                        .help("Sets a file output name")
                        .takes_value(true))
                )
                .arg(Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("Decrease program verbosity"));

    app
}
