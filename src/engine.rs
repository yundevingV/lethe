use std::collections::HashMap;
use std::thread;

use crate::caches;
use crate::catalog;
use crate::docker::{self, DockerUsage};
use crate::model::{CacheRow, CleanResult};
use crate::pnpm;
use crate::runner::{self, find_tool};

pub fn default_rows() -> Vec<CacheRow> {
    catalog::default_rows()
}

pub fn clean(id: &str) -> CleanResult {
    catalog::clean(id)
}

pub fn confirm_text(rows: &[CacheRow]) -> String {
    let picked: Vec<_> = rows.iter().filter(|row| row.selected && row.cleanable).collect();
    let mut lines = vec![
        "다음 항목을 Lethe합니다.".to_string(),
        picked.iter().map(|row| row.title.as_str()).collect::<Vec<_>>().join(", "),
    ];
    if picked.iter().any(|row| row.id == "pnpm") {
        lines.push("pnpm은 어디에서도 쓰이지 않는 파일만 지웁니다.".into());
    }
    if picked.iter().any(|row| row.id == "pnpm-old") {
        lines.push("다른 버전 pnpm 스토어는 폴더째 지웁니다.".into());
    }
    if picked.iter().any(|row| row.id == "docker-images") {
        lines.push("실행 중인 컨테이너가 쓰지 않는 이미지가 삭제됩니다.".into());
    }
    if picked.iter().any(|row| row.id == "playwright") {
        lines.push("Playwright 브라우저는 다음 테스트 때 다시 받습니다.".into());
    }
    lines.push("지운 캐시는 다음 설치나 빌드 때 다시 받습니다.".into());
    lines.join("\n")
}

pub fn scan(existing: &[CacheRow]) -> Vec<CacheRow> {
    let measured = measure(existing);
    let mut rows = existing.to_vec();
    for row in &mut rows {
        row.blocked_reason = None;
        row.bytes = measured.get(&row.id).copied();
    }
    apply_docker(&mut rows);
    for row in &mut rows {
        if row.cleanable && row.bytes.unwrap_or(0) == 0 {
            row.selected = false;
        }
    }
    rows
}

fn measure(existing: &[CacheRow]) -> HashMap<String, u64> {
    let ids: Vec<String> = existing.iter().map(|row| row.id.clone()).collect();
    let mut measured = HashMap::new();
    thread::scope(|scope| {
        let handles: Vec<_> = ids.iter().map(|id| scope.spawn(|| (id.clone(), bytes_for(id)))).collect();
        for handle in handles {
            if let Ok((id, Some(bytes))) = handle.join() {
                measured.insert(id, bytes);
            }
        }
    });
    measured
}

fn bytes_for(id: &str) -> Option<u64> {
    if id == "docker-builder" || id == "docker-images" {
        return None;
    }
    if id == "pnpm" {
        return Some(pnpm_store_bytes());
    }
    if id == "pnpm-old" {
        return Some(pnpm::inactive_bytes());
    }
    if id == "uv" {
        return Some(tool_dir_bytes("uv", &["cache", "dir"], "uv"));
    }
    if id == "pip" {
        return Some(pip_bytes());
    }
    let paths = caches::paths(std::env::consts::OS, id);
    Some(runner::disk_usage(paths))
}

fn pip_bytes() -> u64 {
    let fallback = caches::paths(std::env::consts::OS, "pip");
    let Some(pip) = find_tool("pip3").or_else(|| find_tool("pip")) else {
        return runner::disk_usage(fallback);
    };
    let result = runner::run(&pip, &["cache", "dir"]);
    if result.code == 0 {
        if let Some(path) = runner::absolute_path(&result.output) {
            return runner::disk_usage_abs(&[path.into()]);
        }
    }
    runner::disk_usage(fallback)
}

fn tool_dir_bytes(tool: &str, args: &[&str], id: &str) -> u64 {
    let fallback = caches::paths(std::env::consts::OS, id);
    let Some(bin) = find_tool(tool) else {
        return runner::disk_usage(fallback);
    };
    let result = runner::run(&bin, args);
    if result.code == 0 {
        if let Some(path) = runner::absolute_path(&result.output) {
            return runner::disk_usage_abs(&[path.into()]);
        }
    }
    runner::disk_usage(fallback)
}

fn pnpm_store_bytes() -> u64 {
    let fallback = caches::paths(std::env::consts::OS, "pnpm");
    let Some(pnpm_bin) = find_tool("pnpm") else {
        return runner::disk_usage(fallback);
    };
    let result = runner::run(&pnpm_bin, &["store", "path"]);
    if result.code == 0 {
        if let Some(path) = runner::absolute_path(&result.output) {
            return runner::disk_usage_abs(&[path.into()]);
        }
    }
    runner::disk_usage(fallback)
}

fn apply_docker(rows: &mut [CacheRow]) {
    let probe = DockerProbe::fetch();
    set_docker(rows, "docker-builder", probe.build, &probe.problem);
    set_docker(rows, "docker-images", probe.images, &probe.problem);
}

fn set_docker(rows: &mut [CacheRow], id: &str, bytes: Option<u64>, problem: &Option<String>) {
    let Some(row) = rows.iter_mut().find(|row| row.id == id) else { return };
    row.blocked_reason = problem.clone();
    row.bytes = if problem.is_some() { None } else { bytes };
}

struct DockerProbe {
    images: Option<u64>,
    build: Option<u64>,
    problem: Option<String>,
}

impl DockerProbe {
    fn fetch() -> Self {
        let Some(docker) = find_tool("docker") else {
            return Self { images: None, build: None, problem: Some("docker 명령을 찾지 못했습니다.".into()) };
        };
        let formatted = runner::run(&docker, &["system", "df", "--format", "{{json .}}"]);
        if formatted.code != 0 {
            return Self { images: None, build: None, problem: Some(runner::explain(&formatted.output, "Docker")) };
        }
        let parsed = docker::parse_docker_df(&formatted.output);
        if parsed.images.is_some() || parsed.build.is_some() {
            return from_usage(parsed);
        }
        let table = runner::run(&docker, &["system", "df"]);
        if table.code != 0 {
            return Self { images: None, build: None, problem: Some(runner::explain(&table.output, "Docker")) };
        }
        from_usage(docker::parse_docker_df(&table.output))
    }
}

fn from_usage(usage: DockerUsage) -> DockerProbe {
    DockerProbe { images: usage.images, build: usage.build, problem: None }
}
