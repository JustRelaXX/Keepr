#!/usr/bin/env bash
# Validates the AUR packaging sources. Runs in CI (no makepkg needed).
set -euo pipefail
cd "$(dirname "$0")"

bash -n PKGBUILD
bash -n keepr-bin.install

pkgver_pkgbuild="$(grep -E '^pkgver=' PKGBUILD | cut -d= -f2)"
pkgver_srcinfo="$(grep -P '^\tpkgver = ' .SRCINFO | awk '{print $3}')"
[ "$pkgver_pkgbuild" = "$pkgver_srcinfo" ] \
  || { echo "pkgver mismatch: PKGBUILD=$pkgver_pkgbuild .SRCINFO=$pkgver_srcinfo"; exit 1; }

app_version="$(python3 -c "import json;print(json.load(open('../../src-tauri/tauri.conf.json'))['version'])")"
[ "$pkgver_pkgbuild" = "$app_version" ] \
  || { echo "pkgver $pkgver_pkgbuild != app version $app_version"; exit 1; }

grep -q "releases/download/v\${pkgver}/Keepr_\${pkgver}_amd64.deb" PKGBUILD \
  || { echo "source URL does not track pkgver"; exit 1; }

echo "AUR packaging OK (keepr-bin $pkgver_pkgbuild)"
