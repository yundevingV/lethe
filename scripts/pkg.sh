#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/build-app.sh

VERSION="0.1.0"
PKG="Lethe-${VERSION}.pkg"
IDENT="app.lethe.mac"

rm -f "$PKG"
pkgbuild \
  --component "Lethe.app" \
  --install-location "/Applications" \
  --identifier "$IDENT" \
  --version "$VERSION" \
  "$PKG"

echo "Created $PKG"
echo "설치: 더블클릭 후 응용 프로그램 폴더에 Lethe.app 이 들어갑니다."
