use bevy::{
    camera::visibility::VisibilitySystems,
    prelude::*,
    time::{time_system, TimeSystems},
};

use crate::{
    asset::{EffectAsset, EffectAssetLoader},
    compile_effects,
    properties::EffectProperties,
    register_modifiers,
    spawn::{self, Random},
    tick_spawners,
    time::effect_simulation_time_system,
    update_properties_from_asset, EffectSimulation, ParticleEffect, SpawnerSettings,
};

/// Labels for the Hanabi systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum EffectSystems {
    /// Tick all effect instances to generate particle spawn counts.
    ///
    /// This system runs during the [`PostUpdate`] schedule. Any system which
    /// modifies an effect spawner should run before this set to ensure the
    /// spawner takes into account the newly set values during its ticking.
    TickSpawners,

    /// Compile the effect instances, updating the [`CompiledParticleEffect`]
    /// components.
    ///
    /// [`CompiledParticleEffect`]: crate::CompiledParticleEffect
    CompileEffects,

    /// Update the properties of the effect instance based on the declared
    /// properties in the [`EffectAsset`], updating the associated
    /// [`EffectProperties`] component.
    UpdatePropertiesFromAsset,

    /// Build the effects' kernels and GPU storage, and record this frame's simulation (aurora's
    /// frame commands), after the spawners ticked and the effects compiled.
    Simulate,
}

/// Plugin to add systems related to Hanabi.
#[derive(Debug, Clone, Copy)]
pub struct HanabiPlugin;

impl Plugin for HanabiPlugin {
    fn build(&self, app: &mut App) {
        {
            let type_registry = app.world().resource::<AppTypeRegistry>();
            register_modifiers(type_registry);
        }

        app.init_asset::<EffectAsset>()
            .insert_resource(Random(spawn::new_rng()))
            .init_resource::<Time<EffectSimulation>>()
            .configure_sets(
                PostUpdate,
                (
                    EffectSystems::TickSpawners
                        // This checks the visibility to skip work, so needs to run after
                        // ComputedVisibility was updated.
                        .after(VisibilitySystems::VisibilityPropagate),
                    EffectSystems::CompileEffects,
                    EffectSystems::Simulate
                        .after(EffectSystems::TickSpawners)
                        .after(EffectSystems::CompileEffects),
                ),
            )
            .configure_sets(
                PreUpdate,
                EffectSystems::UpdatePropertiesFromAsset.after(bevy::asset::AssetTrackingSystems),
            )
            .add_systems(
                First,
                effect_simulation_time_system
                    .after(time_system)
                    .in_set(TimeSystems),
            )
            .add_systems(
                PostUpdate,
                (
                    tick_spawners.in_set(EffectSystems::TickSpawners),
                    compile_effects.in_set(EffectSystems::CompileEffects),
                    update_properties_from_asset.in_set(EffectSystems::UpdatePropertiesFromAsset),
                ),
            )
            .add_plugins(crate::aurora::plugin);

        app.init_asset_loader::<EffectAssetLoader>();

        app.register_type::<EffectAsset>()
            .register_type::<ParticleEffect>()
            .register_type::<EffectProperties>()
            .register_type::<SpawnerSettings>()
            .register_type::<Time<EffectSimulation>>();
    }
}
