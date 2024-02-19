use clap::Parser;
use todo_cli::{args::Args, run_command};

fn main() {
    let args = Args::parse();

    run_command(args).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });
}
