#!/bin/sh
# raea with the declared Bevy plugins compiled in (raea-bevy-sdk), for the bench harnesses'
# RAEA_BIN and run.sh. Build: tools/bevy-plugins.sh build sdk/raea/bench/bevy-plugins
exec "$(cd "$(dirname "$0")/../../../.." && pwd)/target/release/raea-bevy-sdk" motion "$@"
