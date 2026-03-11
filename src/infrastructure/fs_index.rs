use std::fs;
use std::path::PathBuf;

use walkdir::WalkDir;

use crate::ui::ResultItem;

const IGNORED_DIRS: [&str; 6] = ["node_modules", "target", ".cache", ".git", ".local", ".npm"];

#[derive(Clone, Default)]
pub struct FileIndex {
    files: Vec<(String, String)>,
}

impl FileIndex {
    pub fn new() -> Self {
        let files = index_home_files();
        Self { files }
    }

    pub fn rank_files(&self, query: &str, limit: usize) -> Vec<ResultItem> {
        let search = query.trim_start_matches("//").to_lowercase();
        let mut results = Vec::new();

        for (name, path) in &self.files {
            if name.to_lowercase().contains(&search) {
                results.push(ResultItem::File {
                    name: name.clone(),
                    path: path.clone(),
                });

                if results.len() >= limit {
                    break;
                }
            }
        }

        results
    }

    pub fn rank_folders(&self, query: &str, limit: usize) -> Vec<ResultItem> {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = PathBuf::from(home);

        let search = query.trim_start_matches('/').to_lowercase();

        let mut results = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.to_lowercase().contains(&search) {
                            results.push(ResultItem::Folder {
                                name: name.to_string(),
                                path: path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }

                if results.len() >= limit {
                    break;
                }
            }
        }

        results
    }
}

fn index_home_files() -> Vec<(String, String)> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut files = Vec::new();

    for entry in WalkDir::new(home)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();

            if name.starts_with('.') {
                return false;
            }

            !IGNORED_DIRS.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path().display().to_string();
            let name = entry.file_name().to_string_lossy().to_string();

            files.push((name, path));
        }
    }

    files
}
