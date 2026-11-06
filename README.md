# PAPERLAND

A top-down open-world exploration game built with Rust + Macroquad.

The world is drawn entirely from typewriter-style text glyphs on recycled-paper colors. Terrain and props are procedural, the map streams as the player explores, and Android touch controls are built in.

## Build locally

```bash
rustup target add aarch64-linux-android
cargo install cargo-apk
cargo apk build --release --target aarch64-linux-android
```

## Android build

Pushing to `main` triggers the GitHub Actions workflow in `.github/workflows/android.yml`. The APK is uploaded as the `paperland-android-apk` artifact.

## Controls

- WASD / arrows: move
- Shift: run
- E: interact
- I: inventory
- M: map
- Q: journal
- Android: on-screen directional pad and action buttons

## Art direction

Warm recycled kraft paper, ink-like monospace glyphs, subtle paper grain, no conventional sprite art. All visible world objects are represented by printable characters.
