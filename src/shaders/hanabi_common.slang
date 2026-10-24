// What generated effect code calls: the PRNG, random distributions, and WGSL's builtins that
// Slang lacks or names differently. Inlined at the top of every generated module.

static uint seed = 0u;

static const float tau = 6.283185307179586476925286766559;

// PCG, https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/
uint pcg_hash(uint input) {
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

// 23 bits of `u` as a float in [0, 1).
float to_float01(uint u) {
    return asfloat((u & 0x007fffffu) | 0x3f800000u) - 1.0;
}

float frand() {
    seed = pcg_hash(seed);
    return to_float01(pcg_hash(seed));
}

float2 frand2() {
    seed = pcg_hash(seed);
    let x = to_float01(seed);
    seed = pcg_hash(seed);
    return float2(x, to_float01(seed));
}

float3 frand3() {
    seed = pcg_hash(seed);
    let x = to_float01(seed);
    seed = pcg_hash(seed);
    let y = to_float01(seed);
    seed = pcg_hash(seed);
    return float3(x, y, to_float01(seed));
}

// Three hashes give the 4 x 24 bits.
float4 frand4() {
    let r0 = pcg_hash(seed);
    let r1 = pcg_hash(r0);
    let r2 = pcg_hash(r1);
    seed = r2;
    let r01 = ((r0 & 0xff000000u) >> 8u) | (r1 & 0x0000ffffu);
    let r12 = ((r1 & 0xffff0000u) >> 8u) | (r2 & 0x000000ffu);
    return float4(to_float01(r0), to_float01(r01), to_float01(r12), to_float01(r2 >> 8u));
}

uint urand() {
    seed = pcg_hash(seed);
    return seed;
}
uint2 urand2() { let x = urand(); return uint2(x, urand()); }
uint3 urand3() { let x = urand(); let y = urand(); return uint3(x, y, urand()); }
uint4 urand4() { let x = urand(); let y = urand(); let z = urand(); return uint4(x, y, z, urand()); }
int irand() { return int(urand()); }
int2 irand2() { return int2(urand2()); }
int3 irand3() { return int3(urand3()); }
int4 irand4() { return int4(urand4()); }
bool brand() { return (urand() & 1u) != 0u; }
bool2 brand2() { return (urand2() & 1u) != 0u; }
bool3 brand3() { return (urand3() & 1u) != 0u; }
bool4 brand4() { return (urand4() & 1u) != 0u; }

float rand_uniform_f(float a, float b) { return a + frand() * (b - a); }
float2 rand_uniform_vec2(float2 a, float2 b) { return a + frand2() * (b - a); }
float3 rand_uniform_vec3(float3 a, float3 b) { return a + frand3() * (b - a); }
float4 rand_uniform_vec4(float4 a, float4 b) { return a + frand4() * (b - a); }

// Box-Muller.
float rand_normal_f(float mean, float std_dev) {
    let u = frand();
    let v = frand();
    return mean + std_dev * sqrt(-2.0 * log(u)) * cos(tau * v);
}
float2 rand_normal_vec2(float2 mean, float2 std_dev) {
    let u = frand();
    let v = frand2();
    return mean + std_dev * sqrt(-2.0 * log(u)) * cos(tau * v);
}
float3 rand_normal_vec3(float3 mean, float3 std_dev) {
    let u = frand();
    let v = frand3();
    return mean + std_dev * sqrt(-2.0 * log(u)) * cos(tau * v);
}
float4 rand_normal_vec4(float4 mean, float4 std_dev) {
    let u = frand();
    let v = frand4();
    return mean + std_dev * sqrt(-2.0 * log(u)) * cos(tau * v);
}

float3 proj(float3 u, float3 v) {
    return dot(v, u) / dot(u, u) * u;
}

// The expression graph's vector constructors, element type inferred as WGSL does.
vector<T, 2> vec2<T : __BuiltinType>(T x, T y) { return vector<T, 2>(x, y); }
vector<T, 3> vec3<T : __BuiltinType>(T x, T y, T z) { return vector<T, 3>(x, y, z); }
vector<T, 4> vec4<T : __BuiltinType>(vector<T, 3> xyz, T w) { return vector<T, 4>(xyz, w); }

// WGSL's 4x8 packing, with its rounding.
uint pack4x8unorm(float4 e) {
    let b = uint4(floor(saturate(e) * 255.0 + 0.5));
    return b.x | (b.y << 8u) | (b.z << 16u) | (b.w << 24u);
}
uint pack4x8snorm(float4 e) {
    let b = uint4(int4(floor(clamp(e, -1.0, 1.0) * 127.0 + 0.5)) & 0xff);
    return b.x | (b.y << 8u) | (b.z << 16u) | (b.w << 24u);
}
float4 unpack4x8unorm(uint u) {
    return float4(uint4(u, u >> 8u, u >> 16u, u >> 24u) & 0xffu) / 255.0;
}
float4 unpack4x8snorm(uint u) {
    let b = int4(uint4(u << 24u, u << 16u, u << 8u, u)) >> 24;
    return max(float4(b) / 127.0, -1.0);
}
