use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(short, long)]
    pub entrypoint: String,
}

pub fn parse_config() -> Config {
    Config::parse()
}
