#!/bin/sh
set -eu
repo="${SOULMATE_REPOSITORY:-veyndrasystems/soulmate}"
version="${SOULMATE_VERSION:-v0.15.0-rc.3}"
os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
case "$os:$arch" in
  linux:x86_64|linux:amd64) target="x86_64-unknown-linux-gnu" ;;
  darwin:arm64) target="aarch64-apple-darwin" ;;
  darwin:x86_64) target="x86_64-apple-darwin" ;;
  *) echo "soulmate: unsupported platform $os/$arch" >&2; exit 1 ;;
esac
if command -v sha256sum >/dev/null 2>&1; then
  checksum=sha256sum
elif command -v shasum >/dev/null 2>&1; then
  checksum=shasum
else
  echo "soulmate: no SHA-256 checksum utility found" >&2
  exit 1
fi
base="https://github.com/$repo/releases/download/$version"
tmp=$(mktemp -d)
cleanup() { find "$tmp" -depth -delete; }
trap cleanup EXIT HUP INT TERM
archive="soulmate-${target}.tar.gz"
curl -fsSL "$base/$archive" -o "$tmp/$archive"
curl -fsSL "$base/$archive.sha256" -o "$tmp/$archive.sha256"
case "$checksum" in
  sha256sum) (cd "$tmp" && sha256sum -c "$archive.sha256") ;;
  shasum) (cd "$tmp" && shasum -a 256 -c "$archive.sha256") ;;
esac
prefix="${SOULMATE_INSTALL_PREFIX:-${HOME:?HOME is required}/.local/bin}"
case "$prefix" in ""|/) echo "soulmate: unsafe install prefix" >&2; exit 1 ;; esac
mkdir -p "$prefix"
tar -xzf "$tmp/$archive" -C "$tmp"
stage="$prefix/.soulmate-install-$$"
install -m 0755 "$tmp/soulmate-${target}" "$stage"
mv -f "$stage" "$prefix/soulmate"
echo "Installed soulmate $version to $prefix/soulmate"
case ":${PATH:-}:" in
  *":$prefix:"*) ;;
  *) echo "Add $prefix to PATH to invoke soulmate by name." ;;
esac
echo "Next: \"$prefix/soulmate\" benchmark"
echo "Then: cd YOUR_PROJECT && \"$prefix/soulmate\" init --mode portable"
