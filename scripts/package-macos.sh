#!/bin/sh
set -eu

rust_sysroot="$(rustc --print sysroot)"
host_target="$(rustc -vV | sed -n 's/^host: //p')"
lld_path="$rust_sysroot/lib/rustlib/$host_target/bin/rust-lld"
app_version="$(node -p "require('./src-tauri/tauri.conf.json').version")"
bundle_dir="src-tauri/target/release/bundle/dmg"
dmg_name="护眼助手_${app_version}_aarch64.dmg"
dmg_path="$bundle_dir/$dmg_name"

if [ ! -x "$lld_path" ]; then
  echo "未找到 Rust 自带链接器：$lld_path" >&2
  exit 1
fi

export RUSTFLAGS="-C linker=$lld_path -C linker-flavor=ld64.lld"
npx tauri build --bundles dmg --ci

if [ ! -f "$dmg_path" ]; then
  echo "未找到预期的 DMG：$dmg_path" >&2
  exit 1
fi

(
  cd "$bundle_dir"
  shasum -a 256 "$dmg_name" > "$dmg_name.sha256"
)

echo "安装包：$dmg_path"
echo "校验值：$dmg_path.sha256"
