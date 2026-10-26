//! History for command mode.
//!
//! Storage and search come from rustyline's history module. Up and Down
//! browse the entries that start with the text typed before the first Up,
//! like vim's command line.
use std::path::{Path, PathBuf};

use rustyline::history::{FileHistory, History, SearchDirection};

/// The most commands kept, in memory and in the history file.
const MAX_LEN: usize = 100;

/// Where Up/Down browsing is in the history.
#[derive(Debug)]
struct Browse {
    /// Index of the entry shown now.
    index: usize,
    /// The text typed before the first Up. Entries must start with it.
    prefix: String,
    /// The text to give back when Down goes past the newest entry.
    draft: String,
}

pub struct CommandHistory {
    entries: FileHistory,
    /// The file to keep the history in. `None` keeps it in memory only.
    path: Option<PathBuf>,
    browse: Option<Browse>,
}

impl std::fmt::Debug for CommandHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandHistory")
            .field("len", &self.entries.len())
            .field("path", &self.path)
            .field("browse", &self.browse)
            .finish()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        let mut entries = FileHistory::new();
        // Neither call can fail for the in-memory part of a FileHistory.
        let _ = entries.set_max_len(MAX_LEN);
        let _ = entries.ignore_dups(true);
        Self {
            entries,
            path: None,
            browse: None,
        }
    }
}

impl CommandHistory {
    /// The default history file: `$XDG_STATE_HOME/sheetui/history`, else
    /// `$HOME/.local/state/sheetui/history`.
    pub fn default_path() -> Option<PathBuf> {
        let state = match std::env::var_os("XDG_STATE_HOME") {
            Some(dir) if !dir.is_empty() => PathBuf::from(dir),
            _ => PathBuf::from(std::env::var_os("HOME")?).join(".local/state"),
        };
        Some(state.join("sheetui").join("history"))
    }

    /// Keep the history in `path`, and load what it holds now.
    ///
    /// History is a convenience, so a file that cannot be read or created
    /// does not stop the app: the history stays in memory.
    pub fn use_file(&mut self, path: &Path) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if path.exists() {
            let _ = self.entries.load(path);
        }
        self.path = Some(path.to_path_buf());
    }

    /// Add a command that was run.
    pub fn add(&mut self, cmd: &str) {
        self.browse = None;
        if cmd.trim().is_empty() {
            return;
        }
        if matches!(self.entries.add(cmd), Ok(true)) {
            if let Some(path) = &self.path {
                let _ = self.entries.append(path);
            }
        }
    }

    /// Stop browsing. The next `older` uses the text shown then as the
    /// prefix.
    pub fn reset(&mut self) {
        self.browse = None;
    }

    /// The next older entry that starts with the prefix, or `None` if there
    /// is none. `shown` is the text in the prompt now.
    pub fn older(&mut self, shown: &str) -> Option<String> {
        let (mut start, prefix) = match &self.browse {
            Some(b) => (b.index.checked_sub(1)?, b.prefix.clone()),
            None => (self.entries.len().checked_sub(1)?, shown.to_string()),
        };
        loop {
            let (index, entry) = self.find(&prefix, start, SearchDirection::Reverse)?;
            // Skip an entry that would not change the prompt.
            if entry != shown {
                let draft = match self.browse.take() {
                    Some(b) => b.draft,
                    None => shown.to_string(),
                };
                self.browse = Some(Browse {
                    index,
                    prefix,
                    draft,
                });
                return Some(entry);
            }
            start = index.checked_sub(1)?;
        }
    }

    /// The next newer entry that starts with the prefix. Past the newest
    /// entry, this gives back the text typed before browsing started.
    /// `None` if not browsing.
    pub fn newer(&mut self, shown: &str) -> Option<String> {
        let browse = self.browse.as_ref()?;
        let mut start = browse.index + 1;
        let prefix = browse.prefix.clone();
        while let Some((index, entry)) = self.find(&prefix, start, SearchDirection::Forward) {
            if entry != shown {
                if let Some(b) = self.browse.as_mut() {
                    b.index = index;
                }
                return Some(entry);
            }
            start = index + 1;
        }
        self.browse.take().map(|b| b.draft)
    }

    /// The nearest entry from `start` in `dir` that starts with `prefix`.
    fn find(&self, prefix: &str, start: usize, dir: SearchDirection) -> Option<(usize, String)> {
        // rustyline's prefix search matches nothing for an empty prefix.
        let found = if prefix.is_empty() {
            self.entries.get(start, dir)
        } else {
            self.entries.starts_with(prefix, start, dir)
        };
        found.ok().flatten().map(|r| (r.idx, r.entry.into_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::CommandHistory;

    fn history(cmds: &[&str]) -> CommandHistory {
        let mut h = CommandHistory::default();
        for cmd in cmds {
            h.add(cmd);
        }
        h
    }

    #[test]
    fn test_older_and_newer_walk_all_entries() {
        let mut h = history(&["w", "sort", "q"]);
        assert_eq!(h.older("").as_deref(), Some("q"));
        assert_eq!(h.older("q").as_deref(), Some("sort"));
        assert_eq!(h.older("sort").as_deref(), Some("w"));
        assert_eq!(h.older("w"), None);
        assert_eq!(h.newer("w").as_deref(), Some("sort"));
        assert_eq!(h.newer("sort").as_deref(), Some("q"));
        assert_eq!(h.newer("q").as_deref(), Some(""));
        assert_eq!(h.newer(""), None);
    }

    #[test]
    fn test_prefix_limits_the_entries() {
        let mut h = history(&["sort B", "w", "sort skip 1", "q"]);
        assert_eq!(h.older("so").as_deref(), Some("sort skip 1"));
        assert_eq!(h.older("sort skip 1").as_deref(), Some("sort B"));
        assert_eq!(h.older("sort B"), None);
        assert_eq!(h.newer("sort B").as_deref(), Some("sort skip 1"));
        assert_eq!(h.newer("sort skip 1").as_deref(), Some("so"));
    }

    #[test]
    fn test_empty_history_and_no_match() {
        let mut h = CommandHistory::default();
        assert_eq!(h.older(""), None);
        assert_eq!(h.newer(""), None);
        let mut h = history(&["w"]);
        assert_eq!(h.older("q"), None);
    }

    #[test]
    fn test_entry_equal_to_the_prompt_is_skipped() {
        let mut h = history(&["sort B", "sort"]);
        assert_eq!(h.older("sort").as_deref(), Some("sort B"));
    }

    #[test]
    fn test_repeated_and_blank_commands_are_not_added() {
        let mut h = history(&["w", "w", "  ", ""]);
        assert_eq!(h.older("").as_deref(), Some("w"));
        assert_eq!(h.older("w"), None);
    }

    #[test]
    fn test_keeps_the_last_entries_only() {
        let cmds: Vec<String> = (0..super::MAX_LEN + 5).map(|i| format!("c{i}")).collect();
        let mut h = CommandHistory::default();
        for cmd in &cmds {
            h.add(cmd);
        }
        let mut oldest = None;
        let mut shown = String::new();
        while let Some(entry) = h.older(&shown) {
            shown = entry.clone();
            oldest = Some(entry);
        }
        assert_eq!(oldest.as_deref(), Some("c5"));
    }

    #[test]
    fn test_file_round_trip() {
        let dir = std::env::temp_dir().join(format!("sheetui_history_{}", std::process::id()));
        let path = dir.join("nested").join("history");
        let _ = std::fs::remove_dir_all(&dir);
        let mut h = CommandHistory::default();
        h.use_file(&path);
        h.add("w");
        h.add("sort B");
        let mut again = CommandHistory::default();
        again.use_file(&path);
        assert_eq!(again.older("").as_deref(), Some("sort B"));
        assert_eq!(again.older("sort B").as_deref(), Some("w"));
        std::fs::remove_dir_all(&dir).expect("clean up");
    }
}
