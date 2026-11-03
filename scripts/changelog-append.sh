#!/usr/bin/env bash
set -euo pipefail
# PR 병합 시 CHANGELOG.md 의 [Unreleased] 아래에 한 줄 추가
ENTRY="${ENTRY:?}"
python3 <<'PY'
import os
from pathlib import Path

entry = os.environ["ENTRY"].strip()
path = Path("CHANGELOG.md")
text = path.read_text(encoding="utf-8")
needle = "## [Unreleased]\n"
if needle not in text:
    raise SystemExit("CHANGELOG.md 에 ## [Unreleased] 섹션이 없습니다.")
if entry in text:
    raise SystemExit(0)
head, tail = text.split(needle, 1)
block = tail.split("\n## ", 1)[0]
rest = tail[len(block) :]
block = block.rstrip("\n")
if block.strip():
    block = block + "\n" + entry + "\n"
else:
    block = "\n" + entry + "\n"
path.write_text(head + needle + block + rest, encoding="utf-8")
print(f"Added: {entry}")
PY
