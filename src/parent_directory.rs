use std::path::PathBuf;

pub struct ParentDirectory {
    path: PathBuf,
    newly_assigned_files: Vec<PathBuf>,
}
