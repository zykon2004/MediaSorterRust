use crate::formatter;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::Path;

const MEDIA_FILE_EXTENSIONS: [&str; 4] = ["mkv", "avi", "mpeg", "mpg"];
const DOWNLOADED_MEDIA_INDICATORS: [&str; 3] = ["720p", "1080p", "2160p"];
const PARENT_IDENTIFIER_EXTENSION: &str = ".parent";
const PREFIX_DELIMINATOR: &str = " | ";

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
    use std::fs::File;
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
    fn parent_series_directory_1<'a>() -> &'a str {
        "Mandalorian 2018"
    }
    #[fixture]
    fn parent_series_directory_2<'a>() -> &'a str {
        "Avatar: The Last Airbender tt9018736"
    }

    #[fixture]
    fn series_root_directory(
        parent_series_directory_1: &str,
        parent_series_directory_2: &str,
    ) -> TempDir {
        let series_root_directory: TempDir = TempDir::new().unwrap();
        let mut parent_directory_path: PathBuf;
        for parent_directory in [parent_series_directory_1, parent_series_directory_2].iter() {
            parent_directory_path = TempDir::with_prefix_in(
                [parent_directory, PREFIX_DELIMINATOR].join(""),
                &series_root_directory,
            )
            .unwrap()
            .into_path();
            File::create(parent_directory_path.join(PARENT_IDENTIFIER_EXTENSION)).unwrap();
        }
        series_root_directory
    }

    #[rstest]
    fn series_root_directory_test(series_root_directory: TempDir) {
        assert!(&series_root_directory.path().exists())
    }
}
