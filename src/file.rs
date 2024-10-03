use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions, remove_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Read, Result, Write};
use zstd;

#[derive(Serialize, Deserialize)]
struct Lock {
    file_name: String,
    begin: usize,
    end: usize,
}

pub fn lock(file_name: String, begin: usize, end: usize) -> Result<bool> {
    {
        let file = OpenOptions::new()
            .read(true)
            .open(&file_name)
            .expect("File couldn't open!");
        let line_count = BufReader::new(file).lines().count();

        if line_count < end {
            return Ok(false);
        }
    }

    let file = OpenOptions::new()
        .read(true)
        .open(".janus/locks.json")
        .expect("Could not open locks.json");
    let mut locks: Vec<Lock> =
        serde_json::from_reader(BufReader::new(file)).expect("Could not read locks.json.");

    // FIXME: Check if the file is being tracked
    for lock in &locks {
        if lock.file_name == *file_name && (lock.begin >= begin || lock.end <= end) {
            // FIXME: A user should only place one lock per file
            println!("A lock already exists for this file!");
            return Ok(false);
        }
    }

    let file_write_copy = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(".janus/locks.json")
        .expect("Could not open locks.json to write");
    let new_lock = Lock {
        file_name,
        begin,
        end,
    };
    locks.push(new_lock);
    serde_json::to_writer(BufWriter::new(file_write_copy), &locks)
        .expect("Failed to write lock to file.");

    Ok(true)
}

#[derive(Eq, PartialEq, PartialOrd)]
struct IndexEntry {
    hash: String,
    path: String,
}

impl std::cmp::Ord for IndexEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.path > other.path {
            std::cmp::Ordering::Greater
        } else if self.path < other.path {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

fn parse_index_entries() -> Vec<IndexEntry> {
    let index = OpenOptions::new()
        .read(true)
        .open(".janus/index")
        .expect("Could not open index file to parse entries!");
    let lines: Vec<String> = BufReader::new(index)
        .lines()
        .map(|line| line.expect("Could not parse line of index file!"))
        .collect();
    let mut index_entries: Vec<IndexEntry> = Vec::new();
    for line in lines {
        let index_entry_split: Vec<&str> = line.split(' ').collect();
        index_entries.push(IndexEntry {
            hash: index_entry_split[0].to_string(),
            path: index_entry_split[1].to_string(),
        });
    }
    index_entries
}

fn write_index_entries(index: Vec<IndexEntry>) {
    let index_file = OpenOptions::new()
        .write(true)
        .open(".janus/index")
        .expect("Could not open index file to write!");
    index_file
        .set_len(0)
        .expect("Could not clear index file before rewriting contents.");
    let mut index_writer = BufWriter::new(index_file);
    for entry in index {
        index_writer
            .write(format!("{} {}\n", entry.hash, entry.path).as_bytes())
            .expect("Could not write to index file");
    }
}

fn add_index_entry(new_entry: IndexEntry, mut index: Vec<IndexEntry>) {
    match index.binary_search(&new_entry) {
        Ok(_) => (),
        Err(pos) => index.insert(pos, new_entry),
    }
    write_index_entries(index);
}

pub fn add(path: &String) -> Result<bool> {
    let index = parse_index_entries();
    let mut new_entry = IndexEntry {
        hash: "".to_string(),
        path: path.to_string(),
    };
    match index.binary_search(&new_entry) {
        Ok(_) => return Ok(false),
        Err(_) => (),
    };
    let content = std::fs::read_to_string(path).expect("Could not read file to string");
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = format!("{:x}", hasher.finalize());
    // FIXME: See about giving this a lifetime to live long enough to be written
    // Would like to avoid this clone
    new_entry.hash = hash.clone();
    add_index_entry(new_entry, index);
    let mut path = format!(".janus/objects/{}", &hash[0..2]);
    fs::create_dir(&path).unwrap_or_else(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
        } else {
            panic!("Could not create directories for new add: {error:?}");
        }
    });

    path.push('/');
    path.push_str(&hash[2..hash.len()]);

    zstd::stream::copy_encode(
        format!("blob {}\0{}", content.len(), content).as_bytes(),
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .expect("Could not create/open file to add!"),
        6,
    )
    .unwrap();
    Ok(true)
}

pub fn status() {
    for entry in parse_index_entries() {
        println!("{}", entry.path);
    }
}

pub fn remove(path: &String) {
    let mut index = parse_index_entries();
    let temp_entry = IndexEntry {
        hash: "".to_string(),
        path: path.to_string(),
    };
    match index.binary_search(&temp_entry) {
        Ok(pos) => drop(index.remove(pos)),
        Err(_) => (),
    };
    write_index_entries(index);
}

pub fn cat_file(hash: &String) {
    let file = OpenOptions::new()
        .read(true)
        .open(format!(
            ".janus/objects/{}/{}",
            &hash[0..2],
            &hash[2..hash.len()]
        ))
        .unwrap();
    let decoder = zstd::Decoder::new(file).unwrap();
    let mut content = String::new();
    BufReader::new(decoder)
        .read_to_string(&mut content)
        .unwrap();
    println!("{}", content.split('\0').nth(1).unwrap());
}

pub fn clean() {
    use std::io::{stdin, stdout};
    print!("Are you sure you want to delete the .janus directory?? [y/N]: ");
    stdout().flush().unwrap();
    let mut answer = String::new();
    stdin().lock().read_line(&mut answer).unwrap();
    if answer.trim().to_lowercase() == "y" || answer.trim().to_lowercase() == "yes" {
        remove_dir_all(".janus").unwrap();
    }
}
