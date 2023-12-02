use anyhow::Result;
use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};
use indoc::indoc;
use log::{LevelFilter};

pub fn print_banner() {
    let banner = indoc! {
r#"


███████╗██╗   ██╗███╗   ███╗    ██╗███╗   ██╗██╗  ██╗
██╔════╝██║   ██║████╗ ████║    ██║████╗  ██║██║ ██╔╝
█████╗  ██║   ██║██╔████╔██║    ██║██╔██╗ ██║█████╔╝
██╔══╝  ╚██╗ ██╔╝██║╚██╔╝██║    ██║██║╚██╗██║██╔═██╗
███████╗ ╚████╔╝ ██║ ╚═╝ ██║    ██║██║ ╚████║██║  ██╗
╚══════╝  ╚═══╝  ╚═╝     ╚═╝    ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝
"#};

    log::info!("{}", format!("{}", banner.green().bold()));
}

pub fn log_banner(text:String) {
    let banner = indoc! {r#"========================================================================================================"#};
    log::info!("{}", format!("{}", banner.green().bold()));
    let text = format!("{:^width$}", text, width = banner.len());
    log::info!("{}", format!("{}", text.green().bold()));
    log::info!("{}", format!("{}", banner.green().bold()));
}

pub fn setup_logger() -> Result<()> {
    let colors = ColoredLevelConfig {
        trace: Color::Cyan,
        debug: Color::Magenta,
        info: Color::Green,
        warn: Color::Blue,
        error: Color::Red,
        ..ColoredLevelConfig::new()
    };

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("evm_ink.log")?)
        .level(log::LevelFilter::Error)
        .level(log::LevelFilter::Warn)
        .level_for("evm_ink_rs", LevelFilter::Info)
        .apply()?;

    Ok(())
}
