# Release recipes

How 0.1.0-canary.2 and canary.3 were built and checked beyond `tools/release.sh` (kept for the next):

- `linux/Dockerfile`: the Ubuntu 24.04 build root (glibc 2.39), run under Rosetta; bun's x64 baseline build, as the
  default bun segfaults there.
- `linux/build.sh VERSION [JOBS]`: in that container, raea with the plugin catalog's features and the two app hosts,
  strip, then `tools/release.sh` under Xvfb (its header has the pitfalls). The VM: `colima start -p x86 --vm-type vz`,
  then `rosetta: true` in ~/.colima/x86/colima.yaml and `vmOpts.vz.rosetta {enabled, binfmt}` in its lima.yaml, and
  `LIMA_HOME=~/.colima/_lima limactl start colima-x86`; docker at unix://~/.colima/x86/docker.sock. Stop it to free a
  deleted target's disk. It runs `tools/cpu-heartbeat.sh`
  mounted as /hb.sh beside the build, so antsy's --stall sees a working compiler.
- `linux/Dockerfile.trixie` and `linux/rl3.sh`: RL3 on Linux, the tarball on a fresh debian:trixie with a new user and
  no network.
- `rl3-mac.sh TARBALL [PONG_DIR]`: RL3 on the Mac, the tarball as a fresh user would run it, sandboxed.
- `../release-web.sh VERSION REF --upload TAG`: the web modules built from REF (core/shape's for @raea/r3f's loadShapes,
  raea-web's for @raea/web) with their SHA256SUMS, attached to the GitHub release TAG: vVERSION beside the archives
  for every release (canary.1 and canary.2 got theirs by hand on 11-06, built from their own shape sources), and between releases a pre-release web-SHA on the public commit SHA that
  publish.sh made from REF (`gh release create web-SHA -R shinyobjectz/raea --target SHA --prerelease` first),
  when core/shape, engine/shape-web or engine/web changed (docs/r3f.feature R3F5).
