// An effect's update kernel: a thread per slot. A live particle ages, runs the update modifiers
// and, still alive, the render modifiers, which shape its TLAS instance row (its axes scaled by
// its size, at its position) and colour its tint. A slot with nothing in it traces nothing.

{{UPDATE_EXTRA}}

{{RENDER_EXTRA}}

[shader("compute")]
[numthreads(64, 1, 1)]
void update(uint3 gid: SV_DispatchThreadID) {
    load_frame();
    let slot = gid.x;
    if (slot >= spawner.capacity) {
        return;
    }
    let state = push.alive[slot];
    if (state == 0u) {
        writeRow(push.rows, slot, noTransform(), noTransform());
        return;
    }

    seed = pcg_hash(slot ^ spawner.seed);
    var particle = push.particles[slot];
    {{AGE_CODE}}
    {{REAP_CODE}}
    {{UPDATE_CODE}}

    if (!is_alive) {
        push.alive[slot] = 0u;
        int at;
        InterlockedAdd(push.state.free_count, 1, at);
        push.free_slots[at] = slot;
        InterlockedAdd(push.state.alive_count, 0xFFFFFFFFu);
        writeRow(push.rows, slot, noTransform(), noTransform());
        return;
    }
{{WRITEBACK_CODE}}

{{INPUTS}}
{{VERTEX_MODIFIERS}}

    // Simulation space to world: the emitter's transform (local simulation) or none.
#if LOCAL_SPACE_SIMULATION
    let e = emitter_transform();
#else
    let e = float4x4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
#endif
    let s = float4x4(float4(axis_x.x * size.x, axis_y.x * size.y, axis_z.x * size.z, position.x),
                     float4(axis_x.y * size.x, axis_y.y * size.y, axis_z.y * size.z, position.y),
                     float4(axis_x.z * size.x, axis_y.z * size.y, axis_z.z * size.z, position.z),
                     float4(0.0, 0.0, 0.0, 1.0));
    let w = mul(e, s);
    Transform now = { w[0], w[1], w[2] };
    let prev = state == 2u ? now : push.last[slot];
    push.last[slot] = now;
    push.alive[slot] = 1u;
    writeRow(push.rows, slot, now, prev);
    // Its colour: what the effect's material takes when it has `instance_tint`.
    writeTint(push.rows, slot, color);
}
