// An effect's storage and frame data, shared by its init and update kernels (src/aurora). One
// effect instance owns its buffers, so a particle's slot is its index everywhere: in the
// particle buffer, the alive flags, the free-slot stack and the block's instance rows.

import instance_rows;

struct Particle {
{{ATTRIBUTES}}};

{{PARENT_PARTICLE}}

{{PROPERTIES}}

// `SimParams` in src/aurora/mod.rs.
struct SimParams {
    // The main camera's frustum planes (xyz, d) and position, world space.
    float4 frustum[6];
    float4 camera_position;
    // Its right, up and back axes, world space.
    float4 camera_axes[3];
    float delta_time;
    float time;
    float virtual_delta_time;
    float virtual_time;
    float real_delta_time;
    float real_time;
    uint num_effects;
    uint pad;
};

// `GpuSpawner` in src/aurora/mod.rs: the emitter this frame.
struct Spawner {
    // Rows of the emitter's world transform (3x4) and of its inverse.
    float4 transform[3];
    float4 inverse_transform[3];
    // Particles to spawn this frame.
    int spawn;
    uint seed;
    uint capacity;
    uint frame;
};

// `GpuEffectState` in src/aurora/mod.rs.
struct EffectState {
    // Entries on the free-slot stack.
    int free_count;
    uint alive_count;
    // Particles ever spawned (wraps): `particle_counter` in init expressions.
    uint particle_counter;
    uint pad;
};

// `EffectPush` in src/aurora/mod.rs.
struct EffectPush {
    Particle* particles;
    uint* free_slots;
    // Per slot: 0 free, 1 alive, 2 born this frame (no motion yet).
    uint* alive;
    // Per slot, last frame's instance transform rows, for motion vectors.
    Transform* last;
    EffectState* state;
    Spawner* spawner;
    SimParams* sim;
    Properties* properties;
    BlockRows rows;
};
[[vk::push_constant]] EffectPush push;

static SimParams sim_params;
static Spawner spawner;
static Properties* properties;
static const uint properties_array_index = 0u;

void load_frame() {
    sim_params = *push.sim;
    spawner = *push.spawner;
    properties = push.properties;
}

float4x4 emitter_transform() {
    return float4x4(spawner.transform[0], spawner.transform[1], spawner.transform[2],
                    float4(0.0, 0.0, 0.0, 1.0));
}

float4x4 emitter_inverse_transform() {
    return float4x4(spawner.inverse_transform[0], spawner.inverse_transform[1],
                    spawner.inverse_transform[2], float4(0.0, 0.0, 0.0, 1.0));
}

float4 transform_position_simulation_to_world(float3 sim_position) {
#if LOCAL_SPACE_SIMULATION
    return mul(emitter_transform(), float4(sim_position, 1.0));
#else
    return float4(sim_position, 1.0);
#endif
}

float3 transform_normal_simulation_to_world(float3 sim_normal) {
#if LOCAL_SPACE_SIMULATION
    // The inverse transpose.
    return mul(transpose((float3x3)emitter_inverse_transform()), sim_normal);
#else
    return sim_normal;
#endif
}

float3 get_camera_position_effect_space() {
#if LOCAL_SPACE_SIMULATION
    return mul(emitter_inverse_transform(), float4(sim_params.camera_position.xyz, 1.0)).xyz;
#else
    return sim_params.camera_position.xyz;
#endif
}

// Rows: the camera's right, up and back axes.
float3x3 get_camera_rotation_effect_space() {
    let r = float3x3(sim_params.camera_axes[0].xyz, sim_params.camera_axes[1].xyz,
                     sim_params.camera_axes[2].xyz);
#if LOCAL_SPACE_SIMULATION
    let inv = (float3x3)emitter_inverse_transform();
    return float3x3(mul(inv, r[0]), mul(inv, r[1]), mul(inv, r[2]));
#else
    return r;
#endif
}
