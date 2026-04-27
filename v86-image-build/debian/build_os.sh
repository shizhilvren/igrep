#!/usr/bin/bash -x
# which python3
PYTHON=$PYTHON_PATH
set -euo pipefail

cd "$(dirname "$0")"

IMAGES=images
OUT_ROOTFS_TAR="$IMAGES"/debian-rootfs.tar
OUT_ROOTFS_FLAT="$IMAGES"/debian-rootfs-flat
OUT_FSJSON="$IMAGES"/debian-fs.json
CONTAINER_NAME=debian-v86
IMAGE_NAME=i386/debian-v86

mkdir -p "$IMAGES"
docker build . --platform linux/386 --rm --tag "$IMAGE_NAME" \
  
docker rm "$CONTAINER_NAME" || true
docker create \
  --platform linux/386 -t -i --name "$CONTAINER_NAME" "$IMAGE_NAME"

docker export "$CONTAINER_NAME" -o "$OUT_ROOTFS_TAR"

# https://github.com/iximiuz/docker-to-linux/issues/19#issuecomment-1242809707
tar -f "$OUT_ROOTFS_TAR" --delete ".dockerenv" || true
