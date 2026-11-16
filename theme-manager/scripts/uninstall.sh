#!/bin/sh
# Removes what install.sh installed. Themes you saved or made global are yours and are kept.
#
#   scripts/uninstall.sh [--prefix PREFIX]
set -eu
prefix=$HOME/.local
while [ $# -gt 0 ]; do
  case "$1" in
    --prefix) prefix=${2:?--prefix needs a directory}; shift 2 ;;
    --prefix=*) prefix=${1#--prefix=}; shift ;;
    *) echo "usage: uninstall.sh [--prefix PREFIX]" >&2; exit 2 ;;
  esac
done
id=org.iad.ThemeEditor
rm -f "$prefix/bin/theme-manager" \
  "$prefix/share/icons/hicolor/scalable/apps/$id.svg" \
  "$prefix/share/applications/$id.desktop" \
  "$prefix/share/metainfo/$id.metainfo.xml"
echo "Removed Theme Manager from $prefix."
