#!/bin/sh
# Installs Theme Manager so the desktop lists it as an app: the binary (which also runs the
# gallery as `theme-manager gallery`), the icon, the .desktop file and the AppStream metadata.
#
#   scripts/install.sh [--prefix PREFIX]
#
# PREFIX defaults to ~/.local. Run `cargo build --release -p iced-adaptive-design-theme-manager`
# first. The bundled themes are inside the binary: nothing else is installed.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
pkg=$(cd "$here/.." && pwd)
root=$(cd "$pkg/.." && pwd)
prefix=$HOME/.local
while [ $# -gt 0 ]; do
  case "$1" in
    --prefix) prefix=${2:?--prefix needs a directory}; shift 2 ;;
    --prefix=*) prefix=${1#--prefix=}; shift ;;
    *) echo "usage: install.sh [--prefix PREFIX]" >&2; exit 2 ;;
  esac
done
id=org.iad.ThemeEditor
bin=$root/target/release/theme-manager
[ -x "$bin" ] || { echo "error: $bin is missing: run cargo build --release -p iced-adaptive-design-theme-manager" >&2; exit 1; }
install -Dm755 "$bin" "$prefix/bin/theme-manager"
install -Dm644 "$pkg/data/$id.svg" "$prefix/share/icons/hicolor/scalable/apps/$id.svg"
install -Dm644 "$pkg/data/$id.desktop" "$prefix/share/applications/$id.desktop"
install -Dm644 "$pkg/data/$id.metainfo.xml" "$prefix/share/metainfo/$id.metainfo.xml"
if command -v update-desktop-database >/dev/null; then
  update-desktop-database "$prefix/share/applications" || true
fi
if command -v gtk-update-icon-cache >/dev/null; then
  gtk-update-icon-cache -qt "$prefix/share/icons/hicolor" || true
fi
echo "Installed Theme Manager to $prefix. Make sure $prefix/bin is on PATH."
