use clap::{Parser, Subcommand};
use std::fs::{ self, OpenOptions };
pub mod file_manager;

#[derive(Debug, Parser)]
#[clap(name="janus", version)]
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
    end: usize
  },
}

fn main() {
  let args = Args::parse();
  
  match &args.command {
    Commands::Lock { file_name, begin, end } => {
      println!("Locking {:?} from lines {:?} to {:?}", file_name, begin, end);
      file_manager::lock(file_name, begin, end);
    }
    
    Commands::Init => {
      println!("Initializing a janus repo here");
      match fs::create_dir_all(".janus/refs") {
        Ok(_res) => println!("Directory created"),
        Err(e) => println!("Could not create dir\n {:?}", e)
      }
      
      match fs::create_dir(".janus/objects") {
        Ok(_res) => println!("Directory created"),
        Err(e) => println!("Could not create dir\n {:?}", e)
      }
      
      match fs::File::create(".janus/locks.json") {
        Ok(_res) => println!("Created locks file"),
        Err(e) => println!("Could not create dir\n {:?}", e)
      }
    }
  }
}
