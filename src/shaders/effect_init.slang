// An effect's init kernel: a thread per particle to spawn this frame, each taking a free slot
// (none left: it spawns nothing) and running the init modifiers on it.

{{INIT_EXTRA}}

[shader("compute")]
[numthreads(64, 1, 1)]
void init(uint3 gid: SV_DispatchThreadID) {
    load_frame();
    if (int(gid.x) >= spawner.spawn) {
        return;
    }
    // Pop a free slot. Every thread of this dispatch only pops, so one that finds the stack
    // empty puts its decrement back and the count ends at zero.
    int free_at;
    InterlockedAdd(push.state.free_count, -1, free_at);
    if (free_at <= 0) {
        InterlockedAdd(push.state.free_count, 1);
        return;
    }
    let slot = push.free_slots[free_at - 1];

    uint particle_counter;
    InterlockedAdd(push.state.particle_counter, 1u, particle_counter);

    seed = pcg_hash(slot ^ spawner.seed);
    let transform = emitter_transform();

    Particle particle = {};
    {{INIT_CODE}}

    {{SIMULATION_SPACE_TRANSFORM_PARTICLE}}

    push.particles[slot] = particle;
    push.alive[slot] = 2u;
    InterlockedAdd(push.state.alive_count, 1u);
}
