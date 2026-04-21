#!/usr/bin/bash -x
# which python3
PYTHON=$PYTHON_PATH
set -euo pipefail

cd "$(dirname "$0")"

IMAGES=images
OUT_ROOTFS_TAR="$IMAGES"/alpine-rootfs.tar
OUT_ROOTFS_FLAT="$IMAGES"/alpine-rootfs-flat
OUT_FSJSON="$IMAGES"/alpine-fs.json
CONTAINER_NAME=alpine-v86
IMAGE_NAME=i386/alpine-v86
ROOT_PATH="../../"
LSP_TAR_PATH="$ROOT_PATH"/lsp_tar/lsp.tar

# https://github.com/iximiuz/docker-to-linux/issues/19#issuecomment-1242809707
tar -f "$OUT_ROOTFS_TAR" --delete ".dockerenv" || true

# 解压 lsp.tar 并重新打包，确保每一级目录都有显式条目
LSP_TMP=$(mktemp -d)
tar -xf "$LSP_TAR_PATH" -C "$LSP_TMP"
find "$LSP_TMP" -mindepth 1 -maxdepth 1 -printf "%f\n" | tar -cf "$IMAGES/all.tar" -C "$LSP_TMP" --files-from=-
rm -rf "$LSP_TMP"

tar --concatenate --file="$IMAGES/all.tar" "$OUT_ROOTFS_TAR"

$PYTHON "$ROOT_PATH"/v86/tools/fs2json.py --zstd --out "$OUT_FSJSON" "$IMAGES/all.tar"

# Note: Not deleting old files here
mkdir -p "$OUT_ROOTFS_FLAT"
$PYTHON "$ROOT_PATH"/v86/tools/copy-to-sha256.py --zstd "$IMAGES/all.tar" "$OUT_ROOTFS_FLAT"

echo "$OUT_ROOTFS_TAR", "$OUT_ROOTFS_FLAT" and "$OUT_FSJSON" created.
