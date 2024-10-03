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
    Add { path: String },
    /// Show staged files
    Status,
    /// Remove a staged file
    Remove { path: String },
    /// Show the contents of a file
    CatFile { hash: String },
    /// Delete the hidden directory
    Clean,
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

            fs::File::create(".janus/index").expect("Could not create index file");
            let mut locks =
                fs::File::create(".janus/locks.json").expect("Could not create locks.json");
            locks
                .write_all(b"[]\n")
                .expect("Could not write to locks file");
        }

        Commands::Add { path } => {
            println!("Adding file {:?}", path);
            file::add(path).expect("Could not add file.");
        }

        Commands::Status => {
            println!("Files currently staged:");
            file::status();
        }

        Commands::Remove { path } => {
            println!("Removing file {} from staging.", path);
            file::remove(path);
        }

        Commands::CatFile { hash } => {
            file::cat_file(hash);
        }

        Commands::Clean => {
            file::clean();
        }
    }
}
