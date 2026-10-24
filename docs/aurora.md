# Hanabi on aurora

Branch `aurora` of slyedoc/bevy_hanabi: Hanabi's front end (effect asset, modifiers, expression
graph, properties, spawners, events) emitting Slang for aurora, the pure-Vulkan ray tracer, in
place of the wgpu render graph. This crate has no renderer: aurora depends on it and hosts the
back end (`bevy_aurora::particles`, in `AuroraDefaultPlugins`), which builds each effect's kernels
from `EffectShaderSources`, owns its storage and writes its particles as TLAS instance rows, so
they are shadowed, reflected and in every view. The candle example and the kernel compile test
live there too.

## Stages

1. Slang codegen: expressions and modifiers emit Slang (`float3`, `lerp`, `frac`, `rsqrt`,
   `select(c, a, b)` is WGSL's `select(b, a, c)`); the init / update / indirect / sort templates
   rewritten in Slang.
2. aurora back end replacing `src/render`: buffers by device address (scalar layout), dispatches
   through aurora's frame commands, generated modules compiled by slangc.
3. Output: instance rows in an `InstanceBlock` (analytic spheres, textured quads with aurora
   materials), last frame's rows for motion vectors.
4. Collision modifier: a ray query per particle against the TLAS, in its environment's mask.
5. Light proxy: an effect drives a `PointLight` (flicker). aurora_files' KitBash importer turns
   flame cards into an emitter plus a light.
6. Later: smoke as a density grid feeding `FogVolume`; `VK_NV_ray_tracing_linear_swept_spheres`
   for sparks and trails.

## Status

- Done: stages 1-3 (Slang codegen, the aurora back end, rows + per-particle colour through
  aurora's `instance_tint`). aurora's `examples/candle.rs` runs it (`--capture DIR` for a windowless
  reference capture); `particles::tests::kernels_compile` there compiles the generated kernels.
- Not yet: GPU spawn events (child effects; warns), fragment-stage modifiers (particle textures,
  alpha cutoff; warns), ribbons, sorting, the old wgpu examples (left in `examples/`, out of the
  manifest until ported), stage 4-6.
- Particle materials: `seen_only` (aurora) when a light stands in for the effect's glow,
  `instance_tint` for per-particle colour (the default material has it).

## References

- NVIDIAGameWorks/dxvk-remix `src/dxvk/shaders/rtx/pass/particles/` (Slang, Zlib): spawn on mesh
  triangles with inherited motion, ring buffer, warp-level sleep skip, sprite sheets, trails.
  Its collision is screen space (last frame's G-buffer); ours is ray queries.
