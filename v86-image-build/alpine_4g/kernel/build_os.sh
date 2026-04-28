#!/usr/bin/bash -x
set -euo pipefail
IMAGE_NAME=i386/alpine-v86
IMAGES=images

docker build -t alpine-kernel-builder .
docker cp alpine-kernel-builder:/home/abuild/packages/main/x86/linux-lts-my_custom-*.apk .

