//! SHIRO's two-field filename/phoneme index with shared padding support.
use std::{
    io::{self, BufRead},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Directory plus indexed filename, before audio/feature suffixes.
    pub stem: PathBuf,
    pub phonemes: Vec<String>,
}
pub fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}
/// Retain literal-space tokenization and padding. Empty phoneme fields contain
/// no tokens. Blank rows are skipped rather than truncating the remaining file.
pub fn read<R: BufRead>(
    mut reader: R,
    directory: &Path,
    left: &[String],
    right: &[String],
) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut line = String::new();
    let mut number = 0;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        number += 1;
        let row = line.trim_end_matches(['\r', '\n']);
        if row.is_empty() {
            continue;
        }
        let Some((filename, phones)) = row.split_once(',') else {
            return Err(format_error(number));
        };
        if filename.is_empty() || phones.contains(',') {
            return Err(format_error(number));
        }
        let middle = if phones.is_empty() {
            Vec::new()
        } else {
            phones.split(' ').map(str::to_owned).collect()
        };
        let capacity = left
            .len()
            .checked_add(middle.len())
            .and_then(|n| n.checked_add(right.len()))
            .ok_or_else(|| io::Error::other("index padding dimension overflow"))?;
        let mut phonemes = Vec::new();
        phonemes
            .try_reserve_exact(capacity)
            .map_err(io::Error::other)?;
        phonemes.extend_from_slice(left);
        phonemes.extend(middle);
        phonemes.extend_from_slice(right);
        entries.try_reserve(1).map_err(io::Error::other)?;
        entries.push(Entry {
            stem: directory.join(filename),
            phonemes,
        });
    }
    Ok(entries)
}
fn format_error(line: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("index format error at line {line}: expected filename,phonemes"),
    )
}
