# PAPERLAND — mobile interface and chunk streaming plan

## 1. Interface goals
- World view uses the maximum available screen area; HUD does not cover the center.
- Visual language: warm fibrous paper, ink outlines, monospaced typography, ASCII/Unicode glyphs.
- Portrait: status at top, world in center, actions and D-pad at bottom.
- Landscape: compact HUD in a corner; controls remain at lower-right.
- Touch targets are at least 36–44 logical pixels.
- Zoom buttons +/−, step 0.12, range 0.55–2.4; camera remains centered on player.
- Desktop/debug: WASD/arrows, E interact, I inventory, M map, Q journal, +/- zoom.

## 2. UI layers
1. Paper background with subtle horizontal grain.
2. World layer in world coordinates: ground, water, roads, vegetation and structures.
3. Player marker @ in rust-red ink.
4. HUD: day/time, HP, backpack, coordinates and compass.
5. Action row: backpack, map, journal and interaction.
6. Touch D-pad and zoom controls.
7. Modal inventory/map/journal with close action.

## 3. Chunk generation and streaming
- Chunk dimensions: 24 × 24 tiles. Integer world coordinates; negative values use Euclidean division/remainder.
- Chunk key: (chunk_x, chunk_y). Local tile coordinate: world_coordinate.rem_euclid(CHUNK_SIZE).
- Generation is deterministic: world seed + world tile coordinates + layer salt feed an integer hash. The same coordinates always produce the same tile.
- Macro terrain is computed from global world coordinates, never chunk-local coordinates. Rivers, paths and biome edges therefore continue seamlessly across chunk borders.
- Load priority: player chunk, its 8 neighbors, then the outer preload ring.
- Active radius: 2 chunks. Preload radius: 3 chunks. Chunks outside the cache radius are evicted from RAM.
- Cache stores compact tile/object IDs, not rendered textures. Glyph rendering only occurs for visible tiles.
- Multi-tile structures use a stable anchor coordinate and a deterministic ID from seed + coordinates.
- Player changes (collected resource, opened door, built object) belong in a sparse per-coordinate overlay separate from generated terrain. Reloading a chunk must preserve these changes.
- Future save format serializes player state and the sparse overlay; chunk generation is independent of frame rate.

## 4. Performance
- Render only tiles inside the camera viewport plus one tile margin.
- Generate each chunk once and cache it.
- Cache target maximum: 49 chunks with radius 3; evict farthest chunks.
- Avoid per-tile heap allocations in the frame loop; glyphs are static strings.
- Zoom changes visible tile count, not world dimensions or generation frequency.

## 5. Delivery stages
1. HTML/CSS/JS layout and touch-control prototype in design/paperland-mobile.html.
2. Port palette, HUD, D-pad, action controls and zoom to Macroquad in src/main.rs.
3. Implement deterministic chunk world and bounded cache.
4. Verify Rust build and Android APK in GitHub Actions.
5. After build passes, polish typography, structures and save data.

## 6. Acceptance criteria
- No empty monochrome field; visible world contains multiple terrain features.
- Chunk borders show no seams or discontinuities.
- Player can cross chunks continuously while camera follows.
- Zoom does not change player coordinates or break movement.
- HUD and controls fit narrow/tall and wide/short screens.
