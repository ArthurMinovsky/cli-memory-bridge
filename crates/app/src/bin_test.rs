use cli_memory_engine::{Storage, RetrievalService};
use std::time::Instant;

fn main() {
    let db_path = "/Users/aminovsky/.cli-memory-bridge-rs/db.sqlite3";
    let start = Instant::now();
    let storage = Storage::open(db_path).unwrap();
    println!("Storage opened in {:?}", start.elapsed());
    
    let start = Instant::now();
    let service = RetrievalService::from_storage(&storage).unwrap();
    println!("RetrievalService built in {:?}", start.elapsed());
}
