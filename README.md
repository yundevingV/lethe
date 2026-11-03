# Lethe

개발 도구 **캐시만** 골라 지우는 macOS / Windows 앱. npm, Go, Cargo, NuGet, Maven, Gradle 등. 경로는 각 사용자 홈(`~` / `%USERPROFILE%`) 기준이고, 없는 캐시는 목록에 나오지 않는다.

UI: [egui](https://github.com/emilk/egui) / eframe

## 설치 (다운로드)

**Rust 없이** 쓰려면 GitHub Releases에서 받는다.

**[Releases → 최신 버전](https://github.com/yundevingV/lethe/releases/latest)**

| 파일 | 플랫폼 | 설치 |
|------|--------|------|
| `Lethe-0.1.0.pkg` | macOS (Apple Silicon) | 더블클릭 → 응용 프로그램에 설치 |
| `Lethe-macOS-arm64.zip` | macOS (Apple Silicon) | zip 풀고 `Lethe.app` 실행 |
| `Lethe-Windows-x64.zip` | Windows 64-bit | zip 풀고 `Lethe.exe` 실행 |

Gatekeeper / SmartScreen 경고가 나올 수 있다. Mac은 **우클릭 → 열기**, Windows는 **추가 정보 → 실행**. 서명은 개발용 수준이며, App Store 배포용 notarization 은 포함되지 않는다.

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

버그·기능 제안·PR은 [CONTRIBUTING.md](CONTRIBUTING.md) 를 본다.
