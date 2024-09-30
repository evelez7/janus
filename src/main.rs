use clap::{Parser, Subcommand};
use std::{
    fs::{self},
    io::Write,
};
pub mod file;

#[derive(Debug, Parser)]
#[clap(name = "janus", version)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a janus repo
    Init,
    /// Lock a file
    Lock {
        /// The name of the file to lock
        file_name: String,
        /// The line to begin the lock
        begin: usize,
        /// The line to end the lock
        end: usize,
    },
    /// Add a file
    Add { file_name: String },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Lock {
            file_name,
            begin,
            end,
        } => {
            println!(
                "Locking {:?} from lines {:?} to {:?}",
                file_name, begin, end
            );
            file::lock(file_name.to_string(), *begin, *end).expect("Could not write lock to file");
        }

        Commands::Init => {
            println!("Initializing a janus repo here");
            fs::create_dir_all(".janus/refs").expect("Could not create refs directory.");
            fs::create_dir(".janus/objects").expect("Could not create objects directory.");

            let mut locks =
                fs::File::create(".janus/locks.json").expect("Could not create locks.json");
            locks
                .write_all(b"[]\n")
                .expect("Could not write to locks file");
        }

        Commands::Add { file_name } => {
            println!("Adding file {:?}", file_name);
            file::add(file_name).expect("Could not add file.");
        }
    }
}
