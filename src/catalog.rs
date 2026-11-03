use crate::caches;
use crate::model::{CacheGroup, CacheRow, CleanResult};
use crate::pnpm;
use crate::runner::{self, find_tool};

fn os() -> &'static str {
    std::env::consts::OS
}

pub struct Spec {
    pub id: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
    pub group: CacheGroup,
    pub cleanable: bool,
    pub default_on: bool,
}

pub const SPECS: &[Spec] = &[
    Spec { id: "docker-disk", title: "Docker 디스크", detail: "이미지와 볼륨이 들어 있는 가상 디스크의 실제 사용량입니다. 이 파일은 지우지 않습니다.", group: CacheGroup::Disk, cleanable: false, default_on: false },
    Spec { id: "docker-builder", title: "Docker 빌드 캐시", detail: "이미지 빌드에 쓴 캐시입니다. 다음 빌드가 더 느려질 수 있습니다.", group: CacheGroup::Docker, cleanable: true, default_on: false },
    Spec { id: "docker-images", title: "Docker 미사용 이미지", detail: "컨테이너가 쓰지 않는 이미지를 지웁니다. 필요한 이미지는 다시 받아야 합니다.", group: CacheGroup::Docker, cleanable: true, default_on: false },
    Spec { id: "npm", title: "npm", detail: "설치 캐시와 npx 캐시입니다. 다음 설치 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: true },
    Spec { id: "uv", title: "uv", detail: "uv 패키지 캐시입니다. 다음 설치 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: true },
    Spec { id: "yarn", title: "yarn", detail: "Yarn 패키지 캐시입니다. 다음 설치 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: true },
    Spec { id: "pnpm", title: "pnpm 스토어", detail: "지금 쓰는 스토어 용량입니다. 정리는 프로젝트에서 안 쓰는 파일만 지웁니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "pnpm-old", title: "pnpm 다른 버전", detail: "지금 PATH의 pnpm이 쓰지 않는 스토어입니다. 그 버전으로 다시 설치하면 받아집니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "pip", title: "pip", detail: "pip 다운로드 캐시입니다. 다음 설치 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: true },
    Spec { id: "brew", title: "Homebrew", detail: "다운로드 캐시입니다. 설치된 프로그램은 그대로입니다.", group: CacheGroup::Install, cleanable: true, default_on: true },
    Spec { id: "maven", title: "Maven", detail: "받아 둔 라이브러리와 래퍼만 지웁니다. settings.xml은 남습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "gradle", title: "Gradle", detail: "받아 둔 라이브러리와 Gradle 배포판입니다. 다음 빌드 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "go", title: "Go 모듈", detail: "받아 둔 모듈입니다. 설치된 Go는 남습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "go-build", title: "Go 빌드 캐시", detail: "컴파일 결과입니다. 다음 빌드 때 다시 만듭니다.", group: CacheGroup::Run, cleanable: true, default_on: false },
    Spec { id: "cargo", title: "Cargo", detail: "크레이트 캐시입니다. rustup과 cargo 실행 파일은 남습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "nuget", title: "NuGet", detail: ".NET 복원 캐시입니다. 다음 빌드 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "composer", title: "Composer", detail: "PHP 패키지 캐시입니다. 다음 설치 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "bundler", title: "Bundler", detail: "Ruby 다운로드 캐시입니다. 설치된 gem은 남습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "poetry", title: "Poetry", detail: "Python 패키지 캐시입니다. 다음 설치 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "coursier", title: "Coursier", detail: "Scala·sbt가 받아 둔 라이브러리입니다. 다음 빌드 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "hex", title: "Hex", detail: "Elixir 패키지 캐시입니다. 다음 설치 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "ccache", title: "ccache", detail: "C/C++ 컴파일 캐시입니다. 다음 빌드 때 다시 만듭니다.", group: CacheGroup::Run, cleanable: true, default_on: false },
    Spec { id: "node-gyp", title: "node-gyp", detail: "네이티브 모듈 빌드 캐시입니다. 다음 빌드 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "playwright", title: "Playwright", detail: "테스트용 브라우저입니다. 다음 테스트 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "typescript", title: "TypeScript", detail: "에디터가 받아 둔 TypeScript 버전입니다. 다음 실행 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "pnpm-cache", title: "pnpm 실행 캐시", detail: "dlx와 메타데이터입니다. 패키지 스토어는 따로 있습니다.", group: CacheGroup::Run, cleanable: true, default_on: false },
    Spec { id: "corepack", title: "Corepack", detail: "Yarn·pnpm 버전 파일입니다. 다음 실행 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "electron", title: "Electron", detail: "Electron 실행 파일 캐시입니다. 다음 빌드 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "prisma", title: "Prisma", detail: "Prisma 엔진 캐시입니다. 다음 실행 때 다시 받습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
    Spec { id: "bun", title: "Bun", detail: "Bun 설치 캐시입니다. 실행 파일은 지우지 않습니다.", group: CacheGroup::Install, cleanable: true, default_on: false },
];

pub fn default_rows() -> Vec<CacheRow> {
    SPECS.iter().map(row_from_spec).collect()
}

pub fn clean(id: &str) -> CleanResult {
    match id {
        "npm" => clean_npm(),
        "yarn" => prefer_tool("yarn", &["cache", "clean"], caches::paths(os(), "yarn")),
        "uv" => prefer_tool("uv", &["cache", "clean"], caches::paths(os(), "uv")),
        "pip" => clean_pip(),
        "brew" | "maven" | "gradle" | "cargo" | "bundler" | "poetry" | "coursier" | "hex" | "ccache"
        | "node-gyp" | "playwright" | "typescript" | "electron" | "prisma" | "bun" => wipe(id),
        "go" => clean_go_mod(),
        "go-build" => clean_go_build(),
        "nuget" => clean_nuget(),
        "composer" => prefer_tool("composer", &["clear-cache"], caches::paths(os(), "composer")),
        "pnpm-cache" => prefer_tool("pnpm", &["cache", "delete"], caches::paths(os(), "pnpm-cache")),
        "corepack" => prefer_tool("corepack", &["cache", "clean"], caches::paths(os(), "corepack")),
        "pnpm" => clean_pnpm(),
        "pnpm-old" => pnpm::empty_inactive(),
        "docker-builder" => clean_docker(&["builder", "prune", "-af"], "빌드 캐시를 정리했습니다."),
        "docker-images" => clean_docker(&["image", "prune", "-af"], "사용 중이지 않은 이미지를 정리했습니다."),
        _ => CleanResult::fail("이 항목은 여기서 지우지 않습니다."),
    }
}

fn row_from_spec(spec: &Spec) -> CacheRow {
    CacheRow {
        id: spec.id.into(),
        title: spec.title.into(),
        detail: spec.detail.into(),
        group: spec.group,
        bytes: None,
        blocked_reason: None,
        selected: spec.default_on,
        status: None,
        status_is_error: false,
        cleanable: spec.cleanable,
    }
}

fn clean_npm() -> CleanResult {
    let paths = caches::paths(os(), "npm");
    let cache = prefer_tool("npm", &["cache", "clean", "--force"], paths);
    if !cache.ok {
        return cache;
    }
    folders(paths, "npx 캐시를 지우지 못했습니다.")
}

fn clean_pip() -> CleanResult {
    if let Some(pip) = find_tool("pip3").or_else(|| find_tool("pip")) {
        if runner::run(&pip, &["cache", "purge"]).code == 0 {
            return CleanResult::ok("정리했습니다.");
        }
    }
    folders(caches::paths(os(), "pip"), "pip 캐시를 지우지 못했습니다.")
}

fn clean_go_mod() -> CleanResult {
    if let Some(go) = find_tool("go") {
        if runner::run(&go, &["clean", "-modcache"]).code == 0 {
            return CleanResult::ok("정리했습니다.");
        }
    }
    wipe("go")
}

fn clean_go_build() -> CleanResult {
    if let Some(go) = find_tool("go") {
        if runner::run(&go, &["clean", "-cache"]).code == 0 {
            return CleanResult::ok("정리했습니다.");
        }
    }
    wipe("go-build")
}

fn clean_nuget() -> CleanResult {
    if let Some(dotnet) = find_tool("dotnet") {
        if runner::run(&dotnet, &["nuget", "locals", "all", "--clear"]).code == 0 {
            return CleanResult::ok("정리했습니다.");
        }
    }
    wipe("nuget")
}

fn wipe(id: &str) -> CleanResult {
    folders(caches::paths(os(), id), "캐시를 지우지 못했습니다.")
}

fn clean_pnpm() -> CleanResult {
    let Some(pnpm) = find_tool("pnpm") else {
        return CleanResult::fail("pnpm 명령을 찾지 못했습니다.");
    };
    let result = runner::run(&pnpm, &["store", "prune"]);
    if result.code != 0 {
        return CleanResult::fail(runner::explain(&result.output, "pnpm"));
    }
    CleanResult::ok("안 쓰는 패키지를 정리했습니다. 스토어 전체는 남아 있습니다.")
}

fn clean_docker(args: &[&str], success: &str) -> CleanResult {
    if args != ["builder", "prune", "-af"] && args != ["image", "prune", "-af"] {
        return CleanResult::fail("허용되지 않은 Docker 명령입니다.");
    }
    let Some(docker) = find_tool("docker") else {
        return CleanResult::fail("docker 명령을 찾지 못했습니다.");
    };
    let result = runner::run(&docker, args);
    if result.code != 0 {
        return CleanResult::fail(runner::explain(&result.output, "Docker"));
    }
    CleanResult::ok(success)
}

fn prefer_tool(name: &str, args: &[&str], fallback: &[&str]) -> CleanResult {
    if let Some(tool) = find_tool(name) {
        if runner::run(&tool, args).code == 0 {
            return CleanResult::ok("정리했습니다.");
        }
    }
    folders(fallback, &format!("{name} 캐시를 지우지 못했습니다."))
}

fn folders(relatives: &[&str], fail: &str) -> CleanResult {
    match runner::empty_dirs(relatives) {
        Ok(()) => CleanResult::ok("정리했습니다."),
        Err(()) => CleanResult::fail(fail),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_disk_is_not_cleanable() {
        assert!(!clean("docker-disk").ok);
    }
}
