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

./video_to_pngs.bash $1
cargo run --release pngs/$pngdest
./pngs_to_mp4.bash dest/pngs/$pngdest $1