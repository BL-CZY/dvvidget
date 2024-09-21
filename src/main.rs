pub mod cli;
pub mod daemon;
pub mod utils;

use clap::Parser;
use cli::args::{self, Args};

fn main() {
    let args = Args::parse();
    args::handle_args(args);
}
