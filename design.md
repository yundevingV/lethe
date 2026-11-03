# Lethe — UI 디자인 노트

구현 전에 여기서 방향을 정하고, 맞추면 `src/ui.rs`에 반영한다.  
스택: **eframe 0.37 / egui** (웹 CSS가 아니라 Rust 위젯·Visuals).

---

## 1. 제품 톤

| 키워드 | 의미 |
|--------|------|
| **Lethe** | 망각의 강 — 지워도 되는 캐시, 다시 받을 수 있는 기억 |
| **안전** | nvm·Docker 설정·node_modules는 건드리지 않음 |
| **명확** | 설치 캐시 vs 실행 캐시, 선택 용량 |

피하고 싶은 인상: 알람성 빨강 과다, “원클릭 전부 삭제”, 게임 UI.

---

## 2. 현재 화면 구조 (as-is)

```
┌─ Header ─────────────────────────────────────┐
│ Lethe + 캐시 합계          [다시 측정]        │
│ [ ] 전체 선택                                 │
│ (배너: 성공/실패)                             │
├─ Body (scroll) ──────────────────────────────┤
│ 가상 디스크                                   │
│ Docker                                        │
│ 설치 캐시  (섹션 체크 + 합계)                 │
│ 실행 캐시                                     │
│   행: [체크] 제목 / 설명 / 용량(우측)         │
├─ Footer ─────────────────────────────────────┤
│ 선택 N GB              [선택 항목 정리]       │
│ 면책 한 줄                                    │
└──────────────────────────────────────────────┘
        ↳ 확인 모달 (정리 전)
```

- 창 기본: **640×880**, 최소 **520×560**
- 폰트: **Apple SD Gothic Neo** (macOS TTC, medium 15 / semibold 제목)
- 선택 행: 연한 파란 배경 + 파란 체크박스 (`rgb 56,132,255`)
- 에러: `rgb(220,70,70)` (Docker 꺼짐, 정리 실패)
- 테마: egui **기본 다크** (커스텀 Visuals 없음)

코드: `src/ui.rs` (`header`, `body`, `section`, `render_row`, `footer`, `colored_checkbox`).

---

## 3. 디자인 방향 후보 (to-be — 여기 적고 골라 쓰기)

### A. 브랜드 / 색

- [ ] **액센트 이름** (예: `lethe-blue`, `river-teal`) — 한 가지만 primary
- [ ] 다크 전용 vs **라이트 모드** 지원 (`ctx.set_visuals`)
- [ ] Lethe **워드마크**만 semibold vs 작은 부제(「개발 캐시」) 추가

제안 팔레트 (초안):

| 역할 | 다크 배경 | 액센트 | 위험 | 보조 텍스트 |
|------|-----------|--------|------|-------------|
| hex | `#1a1d24` | `#3884ff` | `#dc4646` | `#9aa2ae` |

### B. 레이아웃

- [ ] 헤더 **고정** + 본문만 스크롤 (지금도 비슷)
- [ ] **설치 / 실행** 탭으로 나누기 vs 지금처럼 한 스크롤
- [ ] 용량 **막대 그래프** (행 옆 thin bar)
- [ ] Docker 디스크 행: **정보 전용** 배지 (삭제 불가)

### C. 컴포넌트

- [ ] 체크박스: 지금 커스텀 유지 vs egui 기본 + `Visuals`만
- [ ] **Primary 버튼** = 「선택 항목 정리」만 강조, 측정은 secondary
- [ ] 빈 목록 / 측정 중 **스켈레ton** 또는 짧은 애니메이션
- [ ] 확인 모달: 위험 항목(pnpm-old, docker-images) **강조 목록**

### D. 카피 (한글)

- [ ] 섹션 부제 한 줄 (설치 캐시 = 「다음 설치 때 다시 받음」)
- [ ] Docker 디스크 툴팁: **파일 삭제 불가** 이유

---

## 4. 구현 메모 (egui)

- 전역 스타일: `ctx.set_visuals(...)` 또는 `ui.style_mut()` in `install_font` 직후
- 섹션/행: `egui::Frame`, `Margin`, `corner_radius` (지금 8px)
- 라이트: `Visuals::light()` + 같은 accent hex
- Windows: SD Gothic Neo 없으면 **Pretendard/맑은 고딕 fallback** (별도 이슈)

---

## 5. 결정 로그

| 날짜 | 결정 | 비고 |
|------|------|------|
| | | |

---

## 6. 참고

- [README.md](README.md) — 설치·보안
- [CHANGELOG.md](CHANGELOG.md)
- egui theming: [Visuals](https://docs.rs/egui/latest/egui/style/struct.Visuals.html)
