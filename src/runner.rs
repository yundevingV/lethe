use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::caches;

pub struct CommandOutput {
    pub code: i32,
    pub output: String,
}

static LOG_SEQ: AtomicU64 = AtomicU64::new(0);

pub fn home() -> PathBuf {
    let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let fallback = if cfg!(windows) { "C:\\" } else { "/" };
    std::env::var_os(key)
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .filter(|path| path != Path::new(""))
        .unwrap_or_else(|| PathBuf::from(fallback))
}

pub fn expand(relative: &str) -> PathBuf {
    home().join(relative)
}

pub fn find_tool(name: &str) -> Option<PathBuf> {
    search_dirs().into_iter().find_map(|dir| executable_in(&dir, name))
}

pub fn absolute_path(output: &str) -> Option<String> {
    output.lines().map(str::trim).filter(|line| is_absolute_path(line)).last().map(str::to_string)
}

pub fn explain(output: &str, tool: &str) -> String {
    let lower = output.to_lowercase();
    if lower.contains("cannot connect to the docker daemon") || lower.contains("is the docker daemon running") {
        return "Docker Desktop이 꺼져 있습니다. 켠 뒤에 다시 시도해 주세요.".into();
    }
    if lower.contains("permission denied") || lower.contains("eacces") {
        return "권한이 없어 지우지 못했습니다.".into();
    }
    format!("{tool} 정리에 실패했습니다.")
}

pub fn disk_usage(relatives: &[&str]) -> u64 {
    let paths: Vec<_> = relatives.iter().map(|rel| expand(rel)).filter(|path| path.exists()).collect();
    disk_usage_abs(&paths)
}

pub fn disk_usage_abs(paths: &[PathBuf]) -> u64 {
    if paths.is_empty() {
        return 0;
    }
    #[cfg(windows)]
    { return paths.iter().map(|path| folder_bytes(path)).sum(); }
    #[cfg(not(windows))]
    {
        let args: Vec<_> = std::iter::once("-sk".to_string())
            .chain(paths.iter().map(|path| path.to_string_lossy().into_owned()))
            .collect();
        let arg_refs: Vec<_> = args.iter().map(String::as_str).collect();
        parse_du(&run(Path::new("/usr/bin/du"), &arg_refs).output)
    }
}

#[cfg(windows)]
fn folder_bytes(path: &Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(meta) = fs::symlink_metadata(&current) else { continue };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_file() {
            total = total.saturating_add(meta.len());
            continue;
        }
        if let Ok(read) = fs::read_dir(&current) {
            stack.extend(read.filter_map(|entry| entry.ok()).map(|entry| entry.path()));
        }
    }
    total
}

pub fn parse_du(text: &str) -> u64 {
    text.lines()
        .filter_map(|line| line.split([' ', '\t']).next()?.parse::<u64>().ok())
        .map(|kb| kb.saturating_mul(1024))
        .sum()
}

pub fn run(exe: &Path, args: &[&str]) -> CommandOutput {
    let log = std::env::temp_dir().join(format!(
        "lethe-{}-{}.log",
        std::process::id(),
        LOG_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let Ok(file) = fs::File::create(&log) else {
        return CommandOutput { code: 1, output: "로그 파일을 만들지 못했습니다.".into() };
    };
    let Ok(stdout) = file.try_clone() else {
        return CommandOutput { code: 1, output: "로그 파일을 만들지 못했습니다.".into() };
    };
    let mut command = Command::new(exe);
    command
        .args(args)
        .current_dir(home())
        .env("PATH", path_env())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(file));
    let code = match command.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            let _ = fs::write(&log, err.to_string());
            1
        }
    };
    let output = fs::read_to_string(&log).unwrap_or_default();
    let _ = fs::remove_file(&log);
    CommandOutput { code, output: tail(output) }
}

pub fn deletion_allowed(relative: &str) -> bool {
    if !is_deletable(relative) {
        return false;
    }
    let path = expand(relative);
    if !path.exists() {
        return true;
    }
    stable_directory(&path)
}

pub fn empty_dirs(relatives: &[&str]) -> Result<(), ()> {
    for relative in relatives {
        empty_one(relative)?;
    }
    Ok(())
}

fn empty_one(relative: &str) -> Result<(), ()> {
    if !deletion_allowed(relative) {
        return Err(());
    }
    let dir = expand(relative);
    if !dir.is_dir() {
        return Ok(());
    }
    if !stable_directory(&dir) {
        return Err(());
    }
    for entry in fs::read_dir(&dir).map_err(|_| ())? {
        let entry = entry.map_err(|_| ())?;
        fs::remove_dir_all(entry.path()).or_else(|_| fs::remove_file(entry.path())).map_err(|_| ())?;
    }
    Ok(())
}

fn is_deletable(relative: &str) -> bool {
    !relative.contains("..")
        && !relative.starts_with('/')
        && !relative.contains(':')
        && caches::can_wipe(std::env::consts::OS, relative)
}

fn stable_directory(path: &Path) -> bool {
    let Ok(resolved) = path.canonicalize() else { return false };
    let Ok(home_real) = home().canonicalize() else { return false };
    if !resolved.starts_with(&home_real) {
        return false;
    }
    for ancestor in path.ancestors() {
        if ancestor == home() {
            break;
        }
        let Ok(meta) = fs::symlink_metadata(ancestor) else { return false };
        if meta.file_type().is_symlink() {
            return false;
        }
    }
    true
}

fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = platform_dirs();
    for dir in path_dirs() {
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
}

fn platform_dirs() -> Vec<PathBuf> {
    #[cfg(windows)]
    { return windows_tools(); }
    #[cfg(not(windows))]
    {
        let mut dirs = nvm_bins();
        dirs.extend([
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            home().join(".local/bin"),
            home().join(".cargo/bin"),
            home().join("go/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
        ]);
        dirs
    }
}

#[cfg(windows)]
fn windows_tools() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local);
        dirs.push(local.join("pnpm"));
    }
    dirs.push(home().join("AppData/Local/pnpm"));
    dirs.push(home().join("AppData/Roaming/npm"));
    dirs.push(home().join(".cargo/bin"));
    dirs.push(home().join("go/bin"));
    dirs.push(home().join(".local/bin"));
    if let Some(programs) = std::env::var_os("ProgramFiles") {
        dirs.push(PathBuf::from(&programs).join("nodejs"));
        dirs.push(PathBuf::from(programs).join("dotnet"));
    }
    if let Some(root) = std::env::var_os("NVM_HOME") {
        dirs.push(PathBuf::from(&root));
        if let Ok(read) = fs::read_dir(&root) {
            dirs.extend(read.filter_map(|entry| entry.ok()).map(|entry| entry.path()));
        }
    }
    dirs
}

#[cfg(not(windows))]
fn nvm_bins() -> Vec<PathBuf> {
    let root = home().join(".nvm/versions/node");
    let Ok(read) = fs::read_dir(&root) else { return Vec::new() };
    let mut versions: Vec<_> = read.filter_map(|e| e.ok()).map(|e| e.file_name()).collect();
    versions.sort_by(|a, b| version_parts(b).cmp(&version_parts(a)));
    versions.into_iter().map(|name| root.join(name).join("bin")).collect()
}

#[cfg(not(windows))]
fn version_parts(name: &std::ffi::OsStr) -> Vec<u64> {
    name.to_string_lossy().trim_start_matches('v').split('.').map(|part| part.parse().unwrap_or(0)).collect()
}

fn path_env() -> String {
    let sep = if cfg!(windows) { ";" } else { ":" };
    let prefix = search_dirs().iter().map(|path| path.to_string_lossy().into_owned()).collect::<Vec<_>>().join(sep);
    match std::env::var("PATH") {
        Ok(current) if !current.is_empty() => format!("{prefix}{sep}{current}"),
        _ => prefix,
    }
}

fn executable_in(dir: &Path, name: &str) -> Option<PathBuf> {
    let plain = dir.join(name);
    if is_executable(&plain) {
        return Some(plain);
    }
    if cfg!(windows) {
        let exe = dir.join(format!("{name}.exe"));
        if is_executable(&exe) {
            return Some(exe);
        }
    }
    None
}

fn path_dirs() -> Vec<PathBuf> {
    let Ok(path) = std::env::var("PATH") else { return Vec::new() };
    let sep = if cfg!(windows) { ';' } else { ':' };
    path.split(sep).filter(|entry| !entry.is_empty()).map(PathBuf::from).collect()
}

fn is_absolute_path(line: &str) -> bool {
    if line.starts_with('/') || line.starts_with('\\') {
        return true;
    }
    let bytes = line.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn is_executable(path: &Path) -> bool {
    #[cfg(windows)]
    { return path.is_file(); }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).map(|meta| meta.permissions().mode() & 0o111 != 0).unwrap_or(false)
    }
}

fn tail(text: String) -> String {
    if text.len() <= 2000 {
        return text;
    }
    let mut start = text.len() - 2000;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    text[start..].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_du_kilobytes() {
        assert_eq!(parse_du("12\t/tmp/a\n3\t/tmp/b\n"), 15 * 1024);
    }

    #[test]
    fn allowlist_keeps_real_caches_and_refuses_the_rest() {
        assert!(deletion_allowed(".npm/_cacache"));
        assert!(deletion_allowed(".m2/repository"));
        assert!(deletion_allowed(".cargo/registry"));
        assert!(!deletion_allowed(".nvm"));
        assert!(!deletion_allowed(".."));
        assert!(!deletion_allowed(".rustup"));
        assert!(!deletion_allowed(".cargo/bin"));
        assert!(!deletion_allowed("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw"));
        assert!(deletion_allowed("Library/Caches/ms-playwright"));
        assert!(!deletion_allowed("Library/pnpm/store"));
        assert!(!deletion_allowed(".m2"));
    }

    #[test]
    fn reads_windows_drive_paths() {
        assert_eq!(absolute_path("store\nC:\\Users\\a\\.nuget\\packages\n"), Some("C:\\Users\\a\\.nuget\\packages".into()));
    }

    #[test]
    fn explains_stopped_docker() {
        let message = explain("Cannot connect to the Docker daemon", "Docker");
        assert!(message.contains("Docker Desktop이 꺼져"));
    }
}
