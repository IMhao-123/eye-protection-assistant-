#!/bin/sh
set -eu

rust_sysroot="$(rustc --print sysroot)"
host_target="$(rustc -vV | sed -n 's/^host: //p')"
lld_path="$rust_sysroot/lib/rustlib/$host_target/bin/rust-lld"

if [ ! -x "$lld_path" ]; then
  echo "未找到 Rust 自带链接器：$lld_path" >&2
  exit 1
fi

export RUSTFLAGS="-C linker=$lld_path -C linker-flavor=ld64.lld"
npx tauri build --bundles app
