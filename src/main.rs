use std::fs::{File, DirBuilder};
use tempfile::TempDir;
use std::io::Write;
mod formatter;
mod media_file;

fn main() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp_dir_2 = TempDir::new_in(tmp_dir.path()).unwrap();
    let file_path = tmp_dir_2.path().join("my-temporary-note.txt");
    let mut tmp_file = File::create(file_path).unwrap();
    writeln!(tmp_file, "Brian was here. Briefly.").unwrap();
}
