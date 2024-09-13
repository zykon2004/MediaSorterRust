use eyre::{eyre, Result};
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

const MINIMAL_TITLE_LENGTH_WITH_YEAR_SUFFIX: usize = 7;
const UNIFIED_SEPARATOR: &str = ".";
const DEFAULT_TITLE_SEPARATOR: &str = " ";
const FORBIDDEN_SEPARATORS: [&str; 2] = [" ", "_"];
const FORBIDDEN_CHARACTERS: [&str; 2] = [":", ";"];

pub(crate) fn format_series_title_and_file_name(title: &str) -> String {
    let mut formatted_title: String = title.to_lowercase();
    formatted_title = create_unified_separator(&formatted_title);
    formatted_title = remove_the_prefix(&formatted_title, UNIFIED_SEPARATOR);
    formatted_title = remove_year_and_imdb_suffix(&formatted_title, UNIFIED_SEPARATOR);
    formatted_title = remove_forbidden_characters(&formatted_title);
    formatted_title
}

fn replace_many(title: &str, old: &[&str], new: &str) -> String {
    let mut formatted_title: String = title.to_owned();
    for _old in old.iter() {
        formatted_title = formatted_title.replace(_old, new);
    }
    formatted_title
}

fn create_unified_separator(title: &str) -> String {
    replace_many(title, &FORBIDDEN_SEPARATORS, UNIFIED_SEPARATOR)
}

fn remove_forbidden_characters(title: &str) -> String {
    replace_many(title, &FORBIDDEN_CHARACTERS, "")
}

fn remove_the_prefix(title: &str, separator: &str) -> String {
    if let Some(stripped) = title.strip_prefix(&["The", separator].join("")) {
        stripped.to_string()
    } else if let Some(stripped) = title.strip_prefix(&["the", separator].join("")) {
        stripped.to_string()
    } else {
        title.to_owned()
    }
}
lazy_static! {
    static ref IMDB_PATTERN: Regex = Regex::new(r"tt\d+").unwrap();
    static ref RELEASE_YEAR: Regex = Regex::new(r"\b(19[3-9]\d|20[0-3]\d)\b").unwrap();
    static ref SERIES_SEASON_AND_EPISODE: Regex =
        RegexBuilder::new(r"s(?<season>\d\d)e(?<episode>\d\d)")
            .case_insensitive(true)
            .build()
            .unwrap();
}

fn remove_year_and_imdb_suffix(title: &str, separator: &str) -> String {
    let binding = IMDB_PATTERN.replace(title, "");
    let mut formatted_title = binding.trim_end_matches(separator);
    if formatted_title.len() > MINIMAL_TITLE_LENGTH_WITH_YEAR_SUFFIX {
        let suffix = formatted_title.rsplit_once(separator).unwrap().1;
        if RELEASE_YEAR.is_match(suffix) {
            formatted_title = formatted_title
                .trim_end_matches(suffix)
                .trim_end_matches(separator)
        }
    }

    formatted_title.to_string()
}
fn extract_season_and_episode_from_series_filename(filename: &str) -> Result<(String, String)> {
    match SERIES_SEASON_AND_EPISODE.captures(filename) {
        Some(caps) => Ok((caps["season"].to_string(), caps["episode"].to_string())),
        None => Err(eyre!("Didnt find S01E01 pattern"))
    }
}

fn format_series_filename_before_rename(filename: &str, title: &str) -> Result<String> {
    let (season, episode) = match extract_season_and_episode_from_series_filename(&filename) {
        Ok((season, episode)) => (season, episode),
        Err(e) => return Err(e),
    };
    let mut formatted_title = remove_year_and_imdb_suffix(&title, DEFAULT_TITLE_SEPARATOR);
    formatted_title = remove_the_prefix(&formatted_title, DEFAULT_TITLE_SEPARATOR);
    let file_suffix = filename
        .rsplit_once(UNIFIED_SEPARATOR)
        .map(|(_, suffix)| suffix)
        .unwrap();
    Ok(format!("{formatted_title} - {season}x{episode}.{file_suffix}").to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::mismatch_title_and_fileame(
        "The Office tt0386676",
        "The.Mandalorian.S02E02.Chapter.10.1080p.DSNP.WEB-DL.DDP.5.1.Atmos.H.264-PHOENiX.mkv",
        false,
    )]
    #[case::matching_title_and_filename_after_removing(
        "Mandalorian 2018",
        "The.Mandalorian.S02E02.Chapter.10.1080p.DSNP.WEB-DL.DDP.5.1.Atmos.H.264-PHOENiX.mkv",
        true,
    )]
    fn e2e_format_series_title_and_filename(#[case] title: &str, #[case] filename: &str, #[case] expected: bool) {
        let formatted_title = format_series_title_and_file_name(title);
        let formatted_filename = format_series_title_and_file_name(filename);
        assert_eq!(formatted_filename.starts_with(&formatted_title), expected)
    }
    #[rstest]
    #[case::imdb_suffix_is_removed(
        "Avatar: The Last Airbender tt9018736",
        "avatar.the.last.airbender"
    )]
    #[case::non_year_suffix_is_kept("Catch 22", "catch.22")]
    #[case::unified_seperator_applied_suffix_removed("Catch 22_tt5056196", "catch.22")]
    #[case::removed_prefix_and_year_suffix("The Mandalorian 2018", "mandalorian")]
    #[case::series_name_prefix_removed(
        "The.Mandalorian.S02E02.Chapter.10.1080p.DSNP.WEB-DL.DDP.5.1.Atmos.H.264-PHOENiX.mkv",
        "mandalorian.s02e02.chapter.10.1080p.dsnp.web-dl.ddp.5.1.atmos.h.264-phoenix.mkv"
    )]
    #[case::series_name_forbidden_characters_removed(
        "S.W.A.T.2017.S07E10.1080p_HDTV_;;x265-MiNX[TGx]",
        "s.w.a.t.2017.s07e10.1080p.hdtv.x265-minx[tgx]"
    )]
    fn full_title_format(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, format_series_title_and_file_name(input));
    }
    #[rstest]
    #[case::normal(
        "The.Mandalorian.S02E02.Chapter.10.1080p.DSNP.WEB-DL.DDP.5.1.Atmos.H.264-PHOENiX.mkv",
        "The Mandalorian 2018",
        "Mandalorian - 02x02.mkv"
    )]
    #[case::avi_file(
        "S.W.A.T.2017.S07E10.1080p_HDTV_;;x265-MiNX[TGx].avi",
        "S.W.A.T 2017",
        "S.W.A.T - 07x10.avi",
    )]
    fn format_before_rename(#[case] filename: &str, #[case] title: &str, #[case] expected: &str) {
        let result = match format_series_filename_before_rename(filename, title) {
            Ok(result) => result,
            Err(_e) => return,
        };
        assert_eq!(expected, result);
    }
}
