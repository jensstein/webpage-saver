#!/usr/bin/env bash

set -xe

# Download the fonts locally because the loader for Google fonts in the
# next project seems to have trouble downloading fonts if the action
# doesn't complete quickly. The issue could be that the build takes
# longer during CI since both an amd64 and an arm64 image is built. The
# arm64 image is slower due to building with qemu virtualization. It
# seems like a very common issue.
# https://github.com/vercel/next.js/discussions/47009#discussioncomment-12947375
# https://github.com/vercel/next.js/discussions/46012
# https://github.com/vercel/next.js/issues/45080


# https://fonts.google.com/selection/embed
# `curl -L "https://fonts.googleapis.com/css2?family=PT+Sans:ital,wght@0,400;0,700;1,400;1,700&display=swap"`
pushd fonts
curl -L https://fonts.gstatic.com/s/ptsans/v17/jizYRExUiTo99u79D0eEwA.ttf -o pt-sans-italic.ttf
curl -L https://fonts.gstatic.com/s/ptsans/v17/jizdRExUiTo99u79D0e8fOytKA.ttf -o pt-sans-italic-bold.ttf
curl -L https://fonts.gstatic.com/s/ptsans/v17/jizaRExUiTo99u79P0U.ttf -o pt-sans-regular.ttf
curl -L https://fonts.gstatic.com/s/ptsans/v17/jizfRExUiTo99u79B_mh4Ok.ttf -o pt-sans-bold.ttf
