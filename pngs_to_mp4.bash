#!/bin/bash
set -eu

function usage() {
cat <<EOF
Usage:
  ${0} [png destination directory] [original video file]

EOF
exit 1
}

if [ $# -ne 2 ]; then
    usage
fi

set -x

fps=$(bc <<< "scale=4; $(ffprobe -v error -select_streams v -of default=noprint_wrappers=1:nokey=1 -show_entries stream=r_frame_rate $2)")

ffmpeg -r $fps -i $1/image_%05d.png -i $2 \
    -map 0:v -map 1:a \
    -c:v libx264 -c:a aac \
    -pix_fmt yuv420p \
    distorted_$2.mp4