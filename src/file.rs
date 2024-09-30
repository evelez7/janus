use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Result, Write};

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

fn add_index(hash: &String, path: &String) {
    let index_file = OpenOptions::new()
        .write(true)
        .read(true)
        .append(true)
        .open(".janus/index")
        .expect("Could not open index!");
    let mut entries: Vec<IndexEntry> = Vec::new();
    for line_result in BufReader::new(&index_file).lines() {
        let line = line_result.unwrap();
        let entry_split: Vec<&str> = line.split(' ').collect();
        let new_entry = IndexEntry {
            hash: entry_split[0].to_string(),
            path: entry_split[1].to_string(),
        };
        entries.push(new_entry);
    }
    let new_entry = IndexEntry {
        hash: hash.to_string(),
        path: path.to_string(),
    };
    match entries.binary_search(&new_entry) {
        Ok(_) => (),
        Err(index) => entries.insert(index, new_entry),
    }
    index_file
        .set_len(0)
        .expect("Could not clear index file before rewriting contents.");
    let mut index_writer = BufWriter::new(index_file);
    for entry in entries {
        index_writer
            .write(format!("{} {}\n", entry.hash, entry.path).as_bytes())
            .expect("Could not write to index file");
    }
}

pub fn add(path: &String) -> Result<bool> {
    let content = std::fs::read_to_string(path).expect("Could not read file to string");
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = format!("{:x}", hasher.finalize());
    add_index(&hash, path);
    let mut path = format!(".janus/objects/{}", &hash[0..2]);
    fs::create_dir(&path).unwrap_or_else(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
        } else {
            panic!("Could not create directories for new add: {error:?}");
        }
    });

    path.push('/');
    path.push_str(&hash[3..hash.len()]);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Could not create/open file to add");
    file.write_all(content.as_bytes())
        .expect("Could not write file content to new file in object directory.");
    Ok(true)
}
