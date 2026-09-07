# Sprite atlas flicker repro

Pinned to unpatched `hxYuki/bevy` revision
`221e52ae323febfd399bd7d067918fc43648cfb3` (Bevy 0.20.0-dev).
No dependency on disillusion or its local renderer patch.

```sh
cargo run
```

Default: left = `Sprite`, right = `Mesh2d + MeshMaterial2d<SpriteMeshMaterial>`.

- `1`: Sprite only, centered.
- `2`: direct SpriteMeshMaterial only, centered.
- `3`: side-by-side comparison.
- `Tab`: cycle these modes.
- `Space`: pause/resume both animations.

Both paths share the same generated four-frame image, atlas layout, size, frame
number and 200 ms timer. Every frame has the same opaque silhouette, with a
moving white stripe inside. No external assets, scene loading, hot reload,
physics or camera motion are involved. Switching display modes changes only
visibility/position; both animation paths continue updating in sync.

## The difference

```rust
// A: Sprite -> Bevy's automatic material cache -> replacement material handle
sprite.texture_atlas.as_mut().unwrap().index = frame;

// B: an explicitly owned material, retaining the same handle
materials.get_mut(&handle).unwrap().texture_atlas_index = frame;
```

These are not two different shaders. In this Bevy revision, ordinary Sprite
also uses SpriteMeshMaterial. The comparison isolates automatic material
creation/handle replacement versus modifying an existing material during Update.

Expected symptom: the left sprite briefly disappears at frame changes while the
right remains visible. In the pinned engine, SpriteMeshPlugin creates materials
in PostUpdate **after AssetEventSystems**, then assigns the new handle. Render
extraction cannot receive the new material's Added event until the next frame.
Direct changes during Update reach AssetEventSystems in the same frame.

## Headless event-handoff repro

```sh
cargo test -- --nocapture
```

The test runs the real SpriteMeshPlugin, warms up, advances frames, and loops
back after old material handles have been dropped. It checks that the automatic
path is missing its new-material event on the switch frame, while the direct
path has its Modified event. Passing means the timing defect was reproduced;
it is not a screenshot/GPU test. On a fixed engine, the assertion documenting
the missing event should fail.
