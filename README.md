# 개발 캐시

macOS와 Windows용 Rust 앱이다. npm, Go, Cargo, NuGet, Maven, Gradle처럼 개발 캐시 크기를 보여주고, 고른 항목만 지운다. 경로는 실행하는 사람의 홈 기준이고, 없는 캐시는 목록에 나오지 않는다.

화면은 [egui](https://github.com/emilk/egui) / eframe 이다.

## 실행

macOS:

```bash
./scripts/run.sh
```

Windows에서는 Rust가 설치된 뒤 프로젝트 폴더에서:

```bash
cargo build --release
.\target\release\DevCache.exe
```

Windows 캐시는 그 계정의 `%USERPROFILE%` 아래다. npm은 `%LOCALAPPDATA%\npm-cache`, Go 빌드 캐시는 `%LOCALAPPDATA%\go-build`, NuGet은 `%USERPROFILE%\.nuget\packages`, Docker 디스크는 `%LOCALAPPDATA%\Docker\wsl\disk\docker_data.vhdx`를 본다. macOS `.app`은 Windows에서 실행되지 않는다.

처음 실행은 의존성을 받고 릴리스 빌드를 하므로 몇 분 걸린다. 끝나면 창이 열린다.

검사만 돌리려면:

```bash
source "$HOME/.cargo/env"
cargo test
```

## 지우는 것

| 항목 | 동작 |
|---|---|
| npm | `npm cache clean --force`, npx 캐시 폴더 |
| yarn | `yarn cache clean`. 실패하면 Yarn 캐시 폴더만 |
| uv / pip | 각 캐시 삭제 명령. 실패하면 기본 캐시 폴더 |
| Homebrew | `~/Library/Caches/Homebrew` 내용. 설치된 프로그램은 유지 |
| Maven | `~/.m2/repository`와 래퍼. `settings.xml`은 유지 |
| Gradle | `~/.gradle/caches`와 받아 둔 Gradle 배포판 |
| Go | `go clean -modcache`, `go clean -cache`. 설치된 Go는 유지 |
| Cargo | `~/.cargo/registry`, `~/.cargo/git`. rustup과 `~/.cargo/bin`은 유지 |
| NuGet | `dotnet nuget locals all --clear`. 실패하면 패키지 캐시 폴더 |
| Composer / Bundler / Poetry / Coursier / Hex / ccache | 각 캐시 폴더 내용만 |
| Playwright | `~/Library/Caches/ms-playwright` 브라우저. 다음 테스트 때 다시 받음 |
| TypeScript / Corepack / Electron / pnpm 실행 캐시 | 해당 캐시 폴더 내용만. pnpm 스토어는 별도 |
| pnpm | `pnpm store prune`. 지금 쓰는 스토어 폴더는 지우지 않음 |
| pnpm 다른 버전 | 현재 pnpm이 가리키지 않는 스토어 폴더만 삭제 |
| Docker 빌드 캐시 | `docker builder prune -af` |
| Docker 미사용 이미지 | `docker image prune -af` |

정리 버튼은 확인 창 다음에만 동작한다.

## 건드리지 않는 것

- `~/.nvm`, `%APPDATA%\nvm` (설치한 Node)
- `~/.rustup`, `~/.cargo/bin` (Rust 툴체인과 설치한 실행 파일)
- `~/go/bin` (설치된 Go 프로그램)
- SDKMAN, pyenv처럼 언어 버전 설치 폴더
- `~/.docker` 설정
- Docker 가상 디스크 파일 (`Docker.raw`)
- pnpm 스토어 전체
- 프로젝트 안 `node_modules`
- 브라우저 캐시
