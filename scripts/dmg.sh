#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ ! -d Lethe.app ]]; then
  ./scripts/build-app.sh
fi

# shellcheck source=version.sh
source "$(dirname "$0")/version.sh"
DMG="Lethe-${VERSION}.dmg"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp -R Lethe.app "$STAGE/"
ln -s /Applications "$STAGE/Applications"

rm -f "$DMG"
hdiutil create -volname "Lethe" -srcfolder "$STAGE" -ov -format UDZO "$DMG"

echo "Created $DMG"
echo "열기: Lethe.app 을 Applications 로 드래그 (막히면 README macOS 보안 참고)."
