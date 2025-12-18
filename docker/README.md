# Docker cross-build for Raspberry Pi 1 (ARMv6)

This setup builds an `arm-unknown-linux-gnueabihf` binary (ARMv6 + hard-float) and a runnable image for BCM2835-based Pis (Raspberry Pi 1 / Zero).

## Requirements

- Docker with BuildKit + `buildx` enabled.
- Internet access to pull base images and toolchains.

## Build the image

```sh
docker buildx build \
  --platform linux/arm/v6 \
  -f docker/Dockerfile.armv6 \
  -t lifelinetty:armv6 \
  .
```

The build uses a multi-stage pipeline:

- Builder: `rust:<version>-bookworm` plus the Debian `gcc-arm-linux-gnueabihf` cross toolchain. Target is ARMv6 + VFP2 via explicit CPU tuning in `RUSTFLAGS`.
- Runtime: `scratch` with only the compiled binary copied in.

Defaults:

- Target: `arm-unknown-linux-gnueabihf`
- CPU tuning: `-C target-cpu=arm1176jzf-s -C target-feature=+vfp2` for BCM2835 (Pi 1 / Zero)
- Entry: `lifelinetty --run`

## Debug build (optional)

```sh
docker buildx build \
  --platform linux/arm/v6 \
  -f docker/Dockerfile.armv6 \
  --build-arg RUSTFLAGS="-C target-cpu=arm1176jzf-s -C target-feature=+vfp2 -C debuginfo=2" \
  -t lifelinetty:armv6-debug \
  .
```

## Extracting the binary

This Dockerfile produces a tiny `scratch` image intended as an artifact carrier
for release tooling (so we can `docker cp` the cross-compiled binary without
needing emulation).

To extract the binary:

```sh
cid=$(docker create lifelinetty:armv6)
docker cp "$cid:/usr/local/bin/lifelinetty" ./lifelinetty
docker rm "$cid"
```

## Notes

- This build avoids downloading external toolchains (e.g. musl.cc), which makes it more robust in environments where DNS/egress is restricted.
- Because the image is `scratch` and the binary is typically dynamically linked, `docker run lifelinetty:armv6 ...` is not guaranteed to work. Prefer the `.deb`/`.rpm` artifacts for installation.
- If you need async serial, build with `--build-arg RUSTFLAGS=...` and enable the feature: `cargo build --features async-serial ...` (adjust the Dockerfile command as needed).
- The Dockerfile uses cache mounts for cargo registry/git/target to speed up iterative builds.
- Keep `/run/serial_lcd_cache` as the only writable path inside the container (bind-mount if needed). If your adapter lives on another TTY (`/dev/ttyAMA0`, `/dev/ttyS0`, USB serial numbers, etc.), either change the `--device` mapping above or set `device = "..."` in `~/.serial_lcd/config.toml` inside the container volume.
