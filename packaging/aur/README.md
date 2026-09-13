# AUR packaging for `keepr-bin`

These files are the source of the
[AUR package](https://aur.archlinux.org/packages/keepr-bin). The package
downloads the `.deb` built by our GitHub release workflow and unpacks it —
no compilation on the user's machine.

## First-time publishing (maintainer, needs an aur.archlinux.org account)

```sh
git clone ssh://aur@aur.archlinux.org/keepr-bin.git
cp dist/aur/PKGBUILD dist/aur/keepr-bin.install dist/aur/.SRCINFO keepr-bin/
cd keepr-bin
# Fill in the real sha256 of the released .deb first (see below), then:
makepkg --printsrcinfo > .SRCINFO
makepkg -si   # test install locally
git add PKGBUILD keepr-bin.install .SRCINFO
git commit -m "keepr-bin 0.1.2-1"
git push
```

## Updating for a new release

1. Take `pkgver` and `sha256sums_x86_64` from the `aur-bump` artifact (or the
   job summary) of the GitHub release workflow run.
2. Update `PKGBUILD`, set `pkgrel=1`.
3. Regenerate `.SRCINFO` with `makepkg --printsrcinfo > .SRCINFO` on Arch.
4. Test with `makepkg -si`, then commit and push to the AUR repo.

## CI check

`./check.sh` validates the PKGBUILD syntax and that `pkgver` matches
`src-tauri/tauri.conf.json` and `.SRCINFO`. It runs in `Desktop checks`.
