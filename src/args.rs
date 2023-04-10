use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    input_file: PathBuf,
    #[clap(short, long, default_value = "out.h68")]
    out: PathBuf,
    #[clap(short, long)]
    listing: Option<Option<PathBuf>>,
}

impl Args {
    pub fn config(self) -> Config {
        Config {
            input_file: self.input_file,
            listing: self.listing.map(|inner| {
                inner.unwrap_or_else(|| {
                    let mut listing = self.out.clone();
                    listing.set_extension("lis");
                    listing
                })
            }),
            out: self.out,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub input_file: PathBuf,
    pub out: PathBuf,
    pub listing: Option<PathBuf>,
}
