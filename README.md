# PAPERLAND

PAPERLAND is a small top-down exploration game made with Rust and Macroquad. The world is drawn with typewriter-like glyphs on a warm recycled-paper background. Terrain is generated procedurally and streamed around the player.

## Android build

The repository is configured for a reproducible Android ARM64 APK build using Rust 1.95, Android SDK 35, NDK 28.2 and `cargo-quad-apk`.

### GitHub Actions

Every push to `main` starts **Build PAPERLAND Android APK**. When it succeeds, open the run's **Artifacts** section and download `paperland-android-arm64-apk`.

### Local build requirements

- Rust 1.95.0 (selected by `rust-toolchain.toml`)
- Android SDK platform 35 and build-tools 35.0.0
- Android NDK 28.2.13676358
- JDK 17
- `zip`

Set the SDK paths, then run:

```sh
rustup target add aarch64-linux-android
cargo +1.95.0 install --git https://github.com/hoklims/cargo-quad-apk cargo-quad-apk --locked
cargo quad-apk build --release --target aarch64-linux-android
```

The APK is generated in `target/android-artifacts/release/apk/`.

## Controls

- WASD / arrow keys: move
- Left Shift: run
- E: interact
- I: backpack
- M: map
- Q: journal
- Android: on-screen directional pad and action buttons

## Project layout

- `src/main.rs` — game entry point and gameplay
- `assets/` — Android asset directory
- `.github/workflows/android.yml` — Android APK build and verification
- `rust-toolchain.toml` — pinned Rust toolchain
