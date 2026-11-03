# Lethe (레테) — Design Specification

> **"Into the Aurora of Oblivion"**
> system memory & cache cleanup software built with Rust

---

## 1. Executive Summary & Design Vision

Lethe(레테)는 그리스 신화 속 '망각의 강'에서 유래한 시스템 캐시 및 임시 파일 정리 프로그램입니다.

단순한 파일 삭제 툴을 넘어, **시스템의 불필요한 데이터 찌꺼기를 우주 멀리 흩뿌리고 오로라 빛으로 정화하는 몽환적 경험**을 선사하는 것을 목표로 합니다.

* **Core Concept:** Deep Space Void & Glowing Aurora
* **Mood & Atmosphere:** 몽환적인(Dreamy), 네온(Neon), 정숙함(Tranquil), 고성능(Rust-powered)
* **Design Philosophy:**
  * 시각적 지부하 감소 (Glassmorphism으로 투명감 강조)
  * 삭제 인터랙션의 극대화 (별가루 소멸 모션)
  * 직관적이고 미니멀한 UI 레이아웃

---

## 2. Color Palette & Typography

### Color Palette

| Token Name | Color Code / Value | Usage |
| --- | --- | --- |
| `--bg-void` | `#090A0F` | 메인 앱 다크 베이스 배경 |
| `--bg-glass` | `rgba(22, 27, 34, 0.55)` | 카드, 오버레이 패널 배경 (Glassmorphism) |
| `--border-glass` | `rgba(255, 255, 255, 0.12)` | 패널 및 버튼 미세 하이라이트 테두리 |
| `--aurora-cyan` | `#00F2FE` | 메인 오로라 링, 활성화 텍스트, Primary Accent |
| `--aurora-purple` | `#9D50BB` | 성운 그라데이션, 보조 포인트, Hover Highlight |
| `--aurora-pink` | `#FF77A9` | Purge 버튼 펄스, 파동 및 소멸 파티클 효과 |
| `--text-primary` | `#F0F4F8` | 주요 타이틀, 대형 용량 수치 |
| `--text-secondary` | `#8A99AD` | 마이크로 카피, 경로 텍스트, 비활성화 요소 |
| `--dust-gray` | `rgba(160, 175, 200, 0.4)` | 캐시 찌꺼기 입자 (Dust Particles) |

### Typography

* **Headings & Numbers:** `Inter`, `Fira Code` (또는 `JetBrains Mono` — 용량 수치 표기 시)
* **Body & UI Text:** `Pretendard`, `sans-serif`
* **Text Styles:**
  * **Hero Number (Cache Size):** 56px / Bold / Text Glow (`drop-shadow(0 0 12px rgba(0,242,254,0.4))`)
  * **Section Title:** 18px / SemiBold / Inter-letter spacing 0.5px
  * **Micro Copy:** 13px / Regular / Opacity 0.8

이 머신에는 Pretendard / Inter / JetBrains Mono가 없다. 본문은 Apple SD Gothic Neo, 용량 숫자는 SF Mono.

---

## 3. UI/UX Layout Architecture

```
+-------------------------------------------------------------+
|  Lethe                              [ - ] [ ◽ ] [ X ]      |
+-------------------------------------------------------------+
|                                                             |
|                     [ Aurora Core Ring ]                    |
|                        (Orb & Dust)                         |
|                                                             |
|                           1.42 GB                           |
|            "1.42 GB의 성운 잔해(Cache)가 감지됨"             |
|                                                             |
|                 [ Release Into Oblivion ]                   |
|                                                             |
+-------------------------------------------------------------+
|  [ Glass Card ]             |  [ Glass Card ]               |
|  설치 캐시                  |  실행 캐시                    |
|  840 MB                     |  580 MB                       |
+-------------------------------------------------------------+
|  Status                     |  다시 측정                    |
+-------------------------------------------------------------+
```

카테고리 이름은 실제 캐시 구분(설치 / 실행 / Docker / 가상 디스크)을 유지한다. 브라우저 캐시처럼 앱에 없는 분류는 만들지 않는다.

### Layout Components

1. **Header Zone:** 타이틀 `Lethe` (Cyan glow) + 최소화 / 최대화 / 닫기
2. **Hero Visual Zone:** 오로라 링, 용량, 마이크로 카피, Purge 버튼
3. **Category Breakdown:** 반투명 글래스 카드. 카드 안에서 항목 선택
4. **Footer:** 상태 점 + 다시 측정. 설정/상세 탭은 동작이 없어 넣지 않음

---

## 4. Key Interactive Elements

### The Lethe Core

* **Idle:** 링이 천천히 회전. cyan + purple. 용량이 클수록 dust 입자가 늘어남
* **Cleaning:** 회전 빨라짐. pink 펄스. dust가 바깥으로 퍼짐
* **Purified:** cyan shockwave 1회. 수치 `0.00 MB`

### Purge Button

* 기본 문구 `Purge Memory`
* Hover 문구 `Release Into Oblivion`
* 채움 `rgba(0, 242, 254, 0.08)`, 테두리 `rgba(0, 242, 254, 0.3)`, hover glow

egui에는 `backdrop-filter`와 GLSL이 없다. 링·입자·글로우는 Painter로 그린다.

---

## 5. Micro-copy

| 상황 | Lethe 카피 |
| --- | --- |
| 스캔 완료 | `{size}의 성운 잔해(Cache)가 스토리지에 얽혀있습니다.` |
| 청소 시작 | `Lethe의 물길에 시스템 기록을 수놓는 중...` |
| 청소 완료 | `망각의 강으로 찌꺼기를 보냈습니다. {size} 정화 완료.` |
| 오류 | `일부 기억 파편이 응결되어 정화되지 못했습니다.` |

---

## 6. 코드 위치

* UI: `src/ui.rs`
* 창 크기·폰트·테마: `ui.rs` → `run()`, `install_font()`, `apply_theme()`
* 앱 번들: 저장소 루트 `Lethe.app`
