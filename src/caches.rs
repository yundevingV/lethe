struct CachePaths {
    id: &'static str,
    macos: &'static [&'static str],
    windows: &'static [&'static str],
    wipe: bool,
}

const ROWS: &[CachePaths] = &[
    CachePaths { id: "docker-disk", macos: &["Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw"], windows: &["AppData/Local/Docker/wsl/disk/docker_data.vhdx", "AppData/Local/Docker/wsl/data/ext4.vhdx"], wipe: false },
    CachePaths { id: "npm", macos: &[".npm/_cacache", ".npm/_npx"], windows: &["AppData/Local/npm-cache", "AppData/Roaming/npm-cache"], wipe: true },
    CachePaths { id: "yarn", macos: &[".yarn/berry/cache", "Library/Caches/Yarn"], windows: &[".yarn/berry/cache", "AppData/Local/Yarn/Cache"], wipe: true },
    CachePaths { id: "pnpm", macos: &["Library/pnpm/store"], windows: &["AppData/Local/pnpm/store", "AppData/Local/pnpm-store"], wipe: false },
    CachePaths { id: "uv", macos: &[".cache/uv"], windows: &["AppData/Local/uv/cache"], wipe: true },
    CachePaths { id: "pip", macos: &["Library/Caches/pip", ".cache/pip"], windows: &["AppData/Local/pip/Cache"], wipe: true },
    CachePaths { id: "brew", macos: &["Library/Caches/Homebrew"], windows: &[], wipe: true },
    CachePaths { id: "maven", macos: &[".m2/repository", ".m2/wrapper"], windows: &[".m2/repository", ".m2/wrapper"], wipe: true },
    CachePaths { id: "gradle", macos: &[".gradle/caches", ".gradle/wrapper/dists"], windows: &[".gradle/caches", ".gradle/wrapper/dists"], wipe: true },
    CachePaths { id: "go", macos: &["go/pkg/mod"], windows: &["go/pkg/mod"], wipe: true },
    CachePaths { id: "go-build", macos: &["Library/Caches/go-build"], windows: &["AppData/Local/go-build"], wipe: true },
    CachePaths { id: "cargo", macos: &[".cargo/registry", ".cargo/git"], windows: &[".cargo/registry", ".cargo/git"], wipe: true },
    CachePaths { id: "nuget", macos: &[".nuget/packages"], windows: &[".nuget/packages", "AppData/Local/NuGet/v3-cache", "AppData/Local/NuGet/Cache"], wipe: true },
    CachePaths { id: "composer", macos: &[".composer/cache", ".cache/composer", "Library/Caches/composer"], windows: &["AppData/Local/Composer", "AppData/Roaming/Composer/cache", ".composer/cache"], wipe: true },
    CachePaths { id: "bundler", macos: &["Library/Caches/bundler", ".bundle/cache"], windows: &["AppData/Local/bundler/cache", ".bundle/cache"], wipe: true },
    CachePaths { id: "poetry", macos: &["Library/Caches/pypoetry", ".cache/pypoetry"], windows: &["AppData/Local/pypoetry/Cache", ".cache/pypoetry"], wipe: true },
    CachePaths { id: "coursier", macos: &["Library/Caches/Coursier", ".ivy2/cache"], windows: &["AppData/Local/Coursier/cache", ".ivy2/cache"], wipe: true },
    CachePaths { id: "hex", macos: &[".hex/packages"], windows: &[".hex/packages"], wipe: true },
    CachePaths { id: "node-gyp", macos: &["Library/Caches/node-gyp"], windows: &["AppData/Local/node-gyp/Cache"], wipe: true },
    CachePaths { id: "playwright", macos: &["Library/Caches/ms-playwright", "Library/Caches/ms-playwright-go"], windows: &["AppData/Local/ms-playwright", "AppData/Local/ms-playwright-go"], wipe: true },
    CachePaths { id: "typescript", macos: &["Library/Caches/typescript"], windows: &["AppData/Local/typescript"], wipe: true },
    CachePaths { id: "pnpm-cache", macos: &["Library/Caches/pnpm"], windows: &["AppData/Local/pnpm-cache", "AppData/Local/pnpm/cache"], wipe: true },
    CachePaths { id: "corepack", macos: &[".cache/node/corepack"], windows: &["AppData/Local/node/corepack"], wipe: true },
    CachePaths { id: "electron", macos: &["Library/Caches/electron"], windows: &["AppData/Local/electron/Cache"], wipe: true },
    CachePaths { id: "prisma", macos: &[".cache/prisma"], windows: &[".cache/prisma", "AppData/Local/prisma"], wipe: true },
    CachePaths { id: "bun", macos: &[".bun/install/cache"], windows: &[".bun/install/cache"], wipe: true },
    CachePaths { id: "ccache", macos: &[".ccache", "Library/Caches/ccache"], windows: &[".ccache", "AppData/Local/ccache"], wipe: true },
];

pub fn paths(os: &str, id: &str) -> &'static [&'static str] {
    ROWS.iter().find(|row| row.id == id).map(|row| row_paths(os, row)).unwrap_or(&[])
}

pub fn can_wipe(os: &str, relative: &str) -> bool {
    ROWS.iter().filter(|row| row.wipe).any(|row| row_paths(os, row).contains(&relative))
}

fn row_paths<'a>(os: &str, row: &'a CachePaths) -> &'a [&'a str] {
    if os == "windows" { row.windows } else { row.macos }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_and_mac_use_different_cache_folders() {
        assert!(paths("windows", "go-build").contains(&"AppData/Local/go-build"));
        assert!(paths("macos", "go").contains(&"go/pkg/mod"));
        assert!(paths("macos", "go-build").contains(&"Library/Caches/go-build"));
        assert!(paths("windows", "nuget").contains(&".nuget/packages"));
        assert!(paths("windows", "docker-disk").iter().any(|path| path.contains("docker_data.vhdx")));
    }

    #[test]
    fn toolchains_stay_off_the_wipe_list() {
        assert!(can_wipe("macos", ".cargo/registry"));
        assert!(can_wipe("windows", ".nuget/packages"));
        assert!(!can_wipe("macos", ".rustup"));
        assert!(!can_wipe("windows", ".cargo/bin"));
        assert!(!can_wipe("macos", "go/bin"));
        assert!(!can_wipe("windows", ".nvm"));
        assert!(!can_wipe("macos", "Library/pnpm/store"));
        assert!(!can_wipe("windows", "AppData/Local/pnpm/store"));
    }
}
