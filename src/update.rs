use std::fs;
use std::path::PathBuf;

use semver::Version;

use crate::runner;

const API: &str = "https://api.github.com/repos/yundevingV/lethe/releases/latest";
const CHANGELOG_URL: &str = "https://raw.githubusercontent.com/yundevingV/lethe/main/CHANGELOG.md";

pub fn app_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or_else(|_| Version::new(0, 1, 0))
}

pub struct UpdateOffer {
    pub tag: String,
    pub version: Version,
    pub notes: String,
    pub page_url: String,
}

pub fn check_for_update() -> Option<UpdateOffer> {
    let latest = fetch_latest()?;
    if latest.version <= app_version() {
        return None;
    }
    if load_skipped().as_deref() == Some(latest.tag.as_str()) {
        return None;
    }
    Some(latest)
}

pub fn skip_version(tag: &str) -> Result<(), ()> {
    let path = skip_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| ())?;
    }
    fs::write(path, tag).map_err(|_| ())
}

pub fn open_release_page(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

fn skip_path() -> PathBuf {
    runner::home().join(".lethe/skipped_version")
}

fn load_skipped() -> Option<String> {
    fs::read_to_string(skip_path()).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn fetch_latest() -> Option<UpdateOffer> {
    let resp = ureq::get(API)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "Lethe")
        .call()
        .ok()?;
    let body = resp.into_string().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let tag = json.get("tag_name")?.as_str()?.to_string();
    let version = parse_tag(&tag)?;
    let page_url = json
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com/yundevingV/lethe/releases/latest")
        .to_string();
    let api_body = json.get("body").and_then(|v| v.as_str()).unwrap_or("").trim();
    let notes = if api_body.len() >= 20 {
        api_body.to_string()
    } else {
        changelog_section(&version).unwrap_or_else(|| api_body.to_string())
    };
    let notes = if notes.is_empty() {
        format!("{tag} 릴리스가 GitHub에 있습니다.\n\n{page_url}")
    } else {
        notes
    };
    Some(UpdateOffer { tag, version, notes, page_url })
}

fn parse_tag(tag: &str) -> Option<Version> {
    let trimmed = tag.trim().trim_start_matches('v');
    Version::parse(trimmed).ok()
}

fn changelog_section(version: &Version) -> Option<String> {
    let text = ureq::get(CHANGELOG_URL)
        .set("User-Agent", "Lethe")
        .call()
        .ok()?
        .into_string()
        .ok()?;
    let header = format!("## [{version}]");
    let start = text.find(&header)?;
    let rest = &text[start + header.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    let block = rest[..end].trim();
    if block.is_empty() { None } else { Some(block.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_tags() {
        assert_eq!(parse_tag("v0.1.0").unwrap(), Version::new(0, 1, 0));
        assert_eq!(parse_tag("0.2.1").unwrap(), Version::new(0, 2, 1));
    }
}
