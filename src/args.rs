use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add { titles: Vec<String> },
    Done { ids: Vec<usize> },
    Undone { ids: Vec<usize> },
    Remove { ids: Vec<usize> },
    Clear,
    Print,
}
