use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("fast_check")
        .about("快速yara 规则扫描")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("dp")
                .about("扫描指定目录下的所有文件文件")
                .arg(
                    Arg::new("path")
                        .help("指定扫描目录")
                        .default_value("./")
                        .short('p')
                        .long("path")
                        .required(false),
                )
                .arg(
                    Arg::new("thread")
                        .help("指定线程数")
                        .default_value("10")
                        .short('t')
                        .long("thread")
                        .required(false),
                )
                .arg_required_else_help(false),
        )
        .disable_help_subcommand(true)
}
