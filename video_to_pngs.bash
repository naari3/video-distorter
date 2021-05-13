#!/bin/bash
set -eu

function usage() {
cat <<EOF
Usage:
  ${0} [input video name]

EOF
exit 1
}

if [ $# -ne 1 ]; then
    usage
fi

set -x

pngdest=$(basename $1)

mkdir -p pngs/$pngdest

ffmpeg -i $1 -vcodec png pngs/$pngdest/image_%05d.png