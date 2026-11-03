use std::fs;
use std::path::{Path, PathBuf};

use crate::caches;
use crate::model::CleanResult;
use crate::runner::{self, absolute_path};

pub fn active_path() -> Option<PathBuf> {
    let pnpm = runner::find_tool("pnpm")?;
    let result = runner::run(&pnpm, &["store", "path"]);
    if result.code != 0 {
        return None;
    }
    let path = absolute_path(&result.output)?;
    fs::canonicalize(path).ok()
}

pub fn inactive_directories() -> Vec<PathBuf> {
    let active = active_path();
    caches::paths(std::env::consts::OS, "pnpm")
        .iter()
        .flat_map(|relative| inactive_under(relative, active.as_deref()))
        .collect()
}

pub fn inactive_bytes() -> u64 {
    runner::disk_usage_abs(&inactive_directories())
}

pub fn empty_inactive() -> CleanResult {
    let targets = inactive_directories();
    if targets.is_empty() {
        return CleanResult::ok("지울 다른 버전 스토어가 없습니다.");
    }
    for path in &targets {
        if fs::remove_dir_all(path).is_err() {
            return CleanResult::fail("다른 버전 pnpm 스토어를 지우지 못했습니다.");
        }
    }
    CleanResult::ok("다른 버전 pnpm 스토어를 정리했습니다.")
}

fn inactive_under(relative: &str, active: Option<&Path>) -> Vec<PathBuf> {
    let root = runner::expand(relative);
    let Some(root_real) = fs::canonicalize(&root).ok() else { return Vec::new() };
    let Some(entries) = fs::read_dir(&root).ok() else { return Vec::new() };
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| is_inactive(path, &root_real, active))
        .collect()
}

fn is_inactive(path: &Path, root_real: &Path, active: Option<&Path>) -> bool {
    let Ok(real) = fs::canonicalize(path) else { return false };
    if !real.starts_with(root_real) || real == root_real {
        return false;
    }
    active.is_none_or(|active| real != active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_list_excludes_the_active_store() {
        let Some(active) = active_path() else { return };
        let inactive: Vec<_> = inactive_directories().iter().filter_map(|path| fs::canonicalize(path).ok()).collect();
        assert!(!inactive.contains(&active));
    }
}
