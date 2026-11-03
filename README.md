# Lethe

개발 도구 **캐시만** 골라 지우는 macOS / Windows 앱. npm, Go, Cargo, NuGet, Maven, Gradle 등. 경로는 각 사용자 홈(`~` / `%USERPROFILE%`) 기준이고, 없는 캐시는 목록에 나오지 않는다.

UI: [egui](https://github.com/emilk/egui) / eframe — 레이아웃·색 방향은 [design.md](design.md)

## 설치 (다운로드)

**Rust 없이** 쓰려면 GitHub Releases에서 받는다.

**[Releases → 최신 버전](https://github.com/yundevingV/lethe/releases/latest)**

| 파일 | 플랫폼 | 설치 |
|------|--------|------|
| `Lethe-0.1.0.dmg` | macOS (Apple Silicon) | DMG 열기 → `Lethe.app`을 **Applications**로 드래그 (막히면 아래 **macOS 보안**) |
| `Lethe-0.1.0.pkg` | macOS (Apple Silicon) | 더블클릭 → 응용 프로그램에 설치 (막히면 아래 **macOS 보안**) |
| `Lethe-macOS-arm64.zip` | macOS (Apple Silicon) | zip 풀고 `Lethe.app` 실행 (막히면 아래 **macOS 보안**) |
| `Lethe-Windows-x64.zip` | Windows 64-bit | zip 풀고 `Lethe.exe` 실행 (막히면 아래 **Windows 보안**) |

Apple / Microsoft **공식 서명(notarization)은 없다.** 개발용 ad-hoc 서명이라 처음 설치·실행 때 경고가 날 수 있다.

### macOS 보안 (Gatekeeper)

**`.pkg` (`Lethe-0.1.0.pkg`)** — Sequoia 등 최근 macOS에서는 **우클릭 → 열기만으로 pkg가 안 열리는 경우가 많다.**

1. pkg **더블클릭** → 차단되면 **시스템 설정** → **개인정보 보호 및 보안** → **보안** 섹션 **맨 아래**로 내린다.
2. **「Mac을 보호하기 위해 'Lethe-0.1.0.pkg'을(를) 차단했습니다」** 옆 **「그래도 열기」** 를 누른다.
3. pkg 설치를 **다시** 실행한다.

**`Lethe.app` (dmg / zip)** — 첫 실행:

- Finder에서 **Control+클릭(우클릭) → 열기 → 열기** 로 한 번 허용하거나,
- 실행 시도 후 위와 같이 **개인정보 보호 및 보안 → 보안 → 「그래도 열기」** 가 보이면 그걸 사용한다.

### Windows 보안 (SmartScreen)

`Lethe.exe` 실행 시 경고가 뜨면 **추가 정보** → **실행**.

## 사용

1. 앱 실행 → **다시 측정**으로 용량 확인  
2. **전체 선택** 또는 구역별·항목별 체크  
3. **선택 항목 정리** → 확인 후 실행  

설치 캐시(패키지 다운로드)와 실행 캐시(빌드·런타임)는 화면에서 구분된다.

## 소스에서 빌드 (개발자)

```bash
git clone https://github.com/yundevingV/lethe.git
cd lethe
```

macOS:

```bash
chmod +x scripts/*.sh
./scripts/run.sh          # 빌드 + Lethe.app 실행
./scripts/build-app.sh    # Lethe.app 만
./scripts/pkg.sh          # Lethe-0.1.0.pkg
./scripts/dmg.sh          # Lethe-0.1.0.dmg (Applications 드래그용)
```

Windows (PowerShell):

```powershell
cargo build --release
.\target\release\Lethe.exe
```

테스트:

```bash
cargo test
```

## 지우는 것 / 안 지우는 것

| 지우는 것 | 안 지우는 것 |
|-----------|--------------|
| npm, yarn, pnpm, uv, pip, Homebrew 등 **캐시** | nvm, rustup, `~/.cargo/bin`, `~/go/bin` (설치본) |
| Maven/Gradle/Cargo/NuGet 등 **받아 둔 아티팩트** | Docker 가상 디스크, pnpm 스토어 **전체** |
| Docker **빌드 캐시 / 미사용 이미지** (CLI) | `~/.docker` 설정, 프로젝트 `node_modules` |
| Playwright 브라우저, TypeScript 에디터 캐시 등 | 브라우저 캐시 |

정리 전 확인 창이 뜬다. 지운 파일은 다음 설치·빌드 때 다시 받아진다.

## 릴리스 (maintainer)

`v*` 태그를 push하면 GitHub Actions가 Mac pkg·zip, Windows zip을 빌드해 Release에 올린다.

```bash
git tag v0.1.0
git push origin v0.1.0
```

자세한 절차는 [CONTRIBUTING.md](CONTRIBUTING.md) 참고.

## 기여

버그·기능 제안·PR은 [CONTRIBUTING.md](CONTRIBUTING.md) 를 본다. 변경 이력은 [CHANGELOG.md](CHANGELOG.md).
