//! Effect flags, from an effect's generated sources.

use bitflags::bitflags;

bitflags! {
    /// Effect flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LayoutFlags: u32 {
        /// No flags.
        const NONE = 0;
        /// The effect is simulated in local space.
        const LOCAL_SPACE_SIMULATION = (1 << 2);
        /// The effect uses alpha masking instead of alpha blending.
        const USE_ALPHA_MASK = (1 << 3);
        /// The effect is rendered with flipbook texture animation based on the
        /// [`Attribute::SPRITE_INDEX`](crate::Attribute::SPRITE_INDEX) of each particle.
        const FLIPBOOK = (1 << 4);
        /// The effect needs UVs.
        const NEEDS_UV = (1 << 5);
        /// The effect has ribbons.
        const RIBBONS = (1 << 6);
        /// The effects needs normals.
        const NEEDS_NORMAL = (1 << 7);
        /// The effect is fully-opaque.
        const OPAQUE = (1 << 8);
        /// The update kernel emits GPU spawn events to instruct another effect to spawn particles.
        const EMIT_GPU_SPAWN_EVENTS = (1 << 9);
        /// The init kernel spawns particles by consuming GPU spawn events, instead of a single
        /// CPU spawn count.
        const CONSUME_GPU_SPAWN_EVENTS = (1 << 10);
        /// The init or update kernel reads its parent particle.
        const READ_PARENT_PARTICLE = (1 << 11);
        /// The effect reads the particle data in the fragment stage.
        const NEEDS_PARTICLE_FRAGMENT = (1 << 12);
    }
}

impl Default for LayoutFlags {
    fn default() -> Self {
        Self::NONE
    }
}
