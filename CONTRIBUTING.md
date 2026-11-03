# 기여 가이드 (CONTRIBUTING)

Lethe에 기여해 주셔서 감사합니다. 이 문서는 이슈·PR·릴리스 절차를 정리한다.

## 이슈

- 버그: OS, Lethe 버전, 재현 방법, 기대 동작 vs 실제 동작
- 기능: 어떤 캐시/도구를 다루는지, 지워도 안전한 이유
- **실제 캐시 삭제 로그**나 개인 경로 전체를 붙이지 말 것

## 개발 환경

- Rust stable (`rustup`)
- macOS 13+ (앱 번들·pkg 테스트)
- Windows 10/11 (exe 빌드)

```bash
git clone https://github.com/yundevingV/lethe.git
cd lethe
cargo test
```

macOS UI 확인:

```bash
./scripts/run.sh
```

## PR

1. `main`에서 브랜치 생성
2. 변경 범위는 작게 (캐시 allowlist·UI·한 OS 경로 등)
3. `cargo test` 통과
4. PR 본문 **Changelog** 칸에 사용자에게 보일 한 줄 작성 (비우면 제목 사용)
5. 병합되면 [CHANGELOG.md](CHANGELOG.md) `[Unreleased]`에 GitHub Actions가 자동 추가
6. 캐시 삭제 동작을 바꿀 때는 **안전 목록**(`src/caches.rs`, `runner`)과 테스트를 함께 수정

### 코드 위치

| 영역 | 파일 |
|------|------|
| macOS / Windows 경로 | `src/caches.rs` |
| 항목 정의·clean | `src/catalog.rs` |
| 측정·Docker | `src/engine.rs`, `src/docker.rs` |
| UI | `src/ui.rs` |
| 삭제 allowlist | `src/runner.rs` |

새 캐시 항목은 **반드시** `caches.rs`의 wipe 목록과 `catalog.rs` spec을 같이 추가한다. `~/.nvm`, rustup, pnpm 스토어 전체, `node_modules` 는 넣지 않는다.

## 릴리스 (maintainer)

1. `Cargo.toml`의 `version`과 태그 `v0.1.0` 형식 맞추기 (pkg/dmg 파일명은 `scripts/version.sh`가 Cargo.toml에서 읽음)  
2. 커밋 push 후:

```bash
git tag v0.1.0
git push origin v0.1.0
```

3. [Actions](https://github.com/yundevingV/lethe/actions) 에서 **Release** 워크플로 확인  
4. [Releases](https://github.com/yundevingV/lethe/releases) 에 pkg, **dmg**, Mac zip, Windows zip 첨부 확인

수동 업로드 (CI 실패 시):

```bash
./scripts/pkg.sh
./scripts/dmg.sh
gh release create v0.1.0 Lethe-0.1.0.pkg Lethe-0.1.0.dmg --generate-notes
gh release upload v0.1.0 Lethe-Windows-x64.zip   # Windows PC에서 빌드한 zip
```

## 행동 강령

존중·건설적인 리뷰. 괴롭힘·차별 없음.

## 라이선스

기여 코드는 저장소와 같은 조건으로 배포되는 것으로 본다 (별도 LICENSE 파일이 없으면 저장소 기본 정책을 따른다).
