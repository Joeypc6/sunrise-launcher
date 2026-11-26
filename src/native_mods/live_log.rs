//! The log a running native build writes, for the build window's live output. Launcher-only:
//! `build_log.rs` is a copy of the game's file, so this stays apart from it.
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Mutex;

/// The SDK workspace's `build.log` while a build runs; `None` between builds.
static LIVE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Names the log a running build writes; `None` when it ends.
pub(crate) fn set_live(path: Option<PathBuf>) {
    *LIVE.lock().unwrap_or_else(|e| e.into_inner()) = path;
}

/// The last `lines` non-empty lines a running build has written so far, read from the end of
/// its log (at most 64 KiB), else from the launcher's own build log. `None` when neither holds
/// any text.
pub fn live_tail(lines: usize) -> Option<String> {
    let live = LIVE.lock().unwrap_or_else(|e| e.into_inner()).clone();
    live.and_then(|path| tail(&path, lines))
        .or_else(|| tail(&super::log_path(), lines))
}

fn tail(path: &std::path::Path, lines: usize) -> Option<String> {
    const LIMIT: u64 = 64 << 10;
    let mut file = fs::File::open(path).ok()?;
    let skip = file.metadata().ok()?.len().saturating_sub(LIMIT);
    file.seek(SeekFrom::Start(skip)).ok()?;
    let mut bytes = Vec::new();
    file.take(LIMIT).read_to_end(&mut bytes).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    // A read that starts mid-file starts mid-line: drop that partial first line.
    let kept: Vec<&str> = text
        .lines()
        .skip(usize::from(skip > 0))
        .filter(|line| !line.trim().is_empty())
        .collect();
    (!kept.is_empty()).then(|| kept[kept.len().saturating_sub(lines)..].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tail_keeps_the_last_lines_and_skips_blank_ones() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("build.log");
        fs::write(&log, "a\n\nb\r\n   \nc\nd\n").unwrap();
        assert_eq!(tail(&log, 2).as_deref(), Some("c\nd"));
        assert_eq!(tail(&log, 10).as_deref(), Some("a\nb\nc\nd"));
        fs::write(&log, "\n \n").unwrap();
        assert_eq!(tail(&log, 3), None);
        assert_eq!(tail(&dir.path().join("missing.log"), 3), None);
    }

    #[test]
    fn a_long_log_is_read_from_its_end_without_its_partial_first_line() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("build.log");
        let body: String = (0..20_000).map(|n| format!("line {n}\n")).collect();
        fs::write(&log, body).unwrap();
        assert_eq!(
            tail(&log, 3).as_deref(),
            Some("line 19997\nline 19998\nline 19999")
        );
    }
}
