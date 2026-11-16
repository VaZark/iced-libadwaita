#!/bin/sh
# Builds the Theme Manager Flatpak with flatpak-builder.
#
#   scripts/flatpak-build.sh [--install] [--run] [--no-network-check]
#
# --install  also installs it for the current user (flatpak-builder --user --install)
# --run      runs it afterwards (implies --install)
# Output is in build-dir/theme-manager at the repository root; the builder's state (downloads,
# cache) is in .flatpak-builder/. Both are ignored by git.
#
# You need flatpak, flatpak-builder, the Freedesktop 26.08 platform and SDK, and the Rust
# extension:
#   flatpak install flathub org.freedesktop.Platform//26.08 org.freedesktop.Sdk//26.08 \
#       org.freedesktop.Sdk.Extension.rust-stable//26.08
# The build downloads the Rust dependencies, so it needs network access.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
install_flag=
run=
while [ $# -gt 0 ]; do
  case "$1" in
    --install) install_flag=--install; shift ;;
    --run) install_flag=--install; run=1; shift ;;
    *) echo "usage: flatpak-build.sh [--install] [--run]" >&2; exit 2 ;;
  esac
done
command -v flatpak-builder >/dev/null || { echo "error: flatpak-builder is not installed" >&2; exit 1; }
cd "$root"
mkdir -p build-dir
flatpak-builder --user --force-clean ${install_flag:+"$install_flag"} \
  --state-dir=.flatpak-builder build-dir/theme-manager theme-manager/flatpak/org.iad.ThemeEditor.yml
[ -z "$run" ] || exec flatpak run org.iad.ThemeEditor
