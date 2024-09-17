use eyre::{eyre, Result};
use crate::formatter;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::Path;

const MEDIA_FILE_EXTENSIONS: [&str; 4] = ["mkv", "avi", "mpeg", "mpg"];
const DOWNLOADED_MEDIA_INDICATORS: [&str; 3] = ["720p", "1080p", "2160p"];
const PARENT_IDENTIFIER_EXTENSION: &str = ".parent";

fn is_media_file(file_path: &Path) -> bool {
    MEDIA_FILE_EXTENSIONS
        .iter()
        .any(|extension| file_path.extension().unwrap_or(OsStr::new("")) == *extension)
}

fn is_downloaded(file_path: &Path) -> bool {
    DOWNLOADED_MEDIA_INDICATORS.iter().any(|indicator| {
        file_path
            .file_stem()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .contains(indicator)
    })
}

fn is_downloaded_media_file(file_path: &Path) -> bool {
    is_downloaded(file_path) && is_media_file(file_path)
}

fn is_downloaded_media_directory(directory: &Path) -> bool {
    if !directory.is_dir() && is_downloaded(directory) {
        return false;
    }
    for file in read_dir(directory).unwrap() {
        let file_path = file.unwrap().path();
        if file_path.is_file() && is_media_file(&file_path) {
            return true;
        }
    }
    false
}

fn is_series_file(file_path: &Path) -> bool {
    let file_path_string = String::from(file_path.file_stem().unwrap().to_string_lossy());

    is_downloaded_media_file(file_path)
        && formatter::extract_season_and_episode_from_series_filename(&file_path_string).is_ok()
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::fs::{create_dir, File};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[rstest]
    #[case::downloaded_file_containing_pattern(
        Path::new("The.Mandalorian.S02E02.Chapter.10.720p.WEB-DL.DDP.5.1.Atmos.H.264-PHOENiX.mkv"),
        true
    )]
    #[case::download_media_but_not_series(
        Path::new(
            "The.Ministry.of.Ungentlemanly.Warfare.2024.1080p.AMZN.WEBRip.1400MB-GalaxyRG.avi"
        ),
        false
    )]
    #[case::jpeg_file(Path::new("1.jpeg"), false)]
    #[case::not_a_downloaded_file(Path::new("Our Wedding 2019.mkv"), false)]
    fn test_is_series_file(#[case] title: &Path, #[case] expected: bool) {
        assert_eq!(is_series_file(title), expected)
    }
    #[fixture]
    fn temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create a temporary directory")
    }
    #[fixture]
    fn parent_series_directory_1<'a>() -> &'a str {
        "Mandalorian 2018"
    }
    #[fixture]
    fn parent_series_directory_2<'a>() -> &'a str {
        "Avatar: The Last Airbender tt9018736"
    }
    #[fixture]
    fn series_root_directory<'b>(
        temp_dir: TempDir,
        parent_series_directory_1: &str,
        parent_series_directory_2: &str,
    ) -> Result<TempDir> {
        let series_root_directory: TempDir = TempDir::new_in(temp_dir.path())?;
        for parent_directory in [parent_series_directory_1, parent_series_directory_2].iter() {
            let directory: PathBuf = series_root_directory.as_ref().join(*parent_directory);
            File::create(&directory.join(PARENT_IDENTIFIER_EXTENSION))?;
        }
        Ok(series_root_directory)
    }
    #[rstest]
    fn test_using_temp_dir(series_root_directory: &Result<TempDir>) {
        // Get the path of the temporary directory
        let path: PathBuf = series_root_directory.as_ref().unwrap().path().to_path_buf();
        assert!(path.exists());
        assert!(path.is_dir());

    }

    #[rstest]
    fn another_test_using_temp_dir(temp_dir: TempDir) {
        // This test will also receive a new temp directory
        let path: PathBuf = temp_dir.path().to_path_buf();
        assert!(path.exists());

        // Add your test logic here
    }
}
