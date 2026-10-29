#!/bin/bash
# Fail before compiling Rust when the selected Xcode cannot build the native launcher.
set -euo pipefail
version=$(xcodebuild -version)
printf '%s\n' "$version"
major=$(printf '%s\n' "$version" | sed -nE 's/^Xcode ([0-9]+).*/\1/p')
if ! [[ "$major" =~ ^[0-9]+$ ]] || (( major < 26 )); then
  echo '::error::The native iOS launcher requires Xcode 26 or newer for Liquid Glass. Use macos-26, or select Xcode 26+ on the MAC_RUNNER override.' >&2
  exit 1
fi
sdk=$(xcrun --sdk iphoneos --show-sdk-version)
printf 'iPhoneOS SDK: %s\n' "$sdk"
major=${sdk%%.*}
if ! [[ "$major" =~ ^[0-9]+$ ]] || (( major < 26 )); then
  echo '::error::The selected iPhoneOS SDK must be 26 or newer. Check DEVELOPER_DIR and xcode-select.' >&2
  exit 1
fi
