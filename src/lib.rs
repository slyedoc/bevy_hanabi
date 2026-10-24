#![deny(
    warnings,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    clippy::suboptimal_flops,
    clippy::imprecise_flops,
    clippy::branches_sharing_code,
    clippy::suspicious_operation_groupings,
    clippy::useless_let_if_seq
)]
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//! 🎆 Hanabi -- a GPU particle system plugin for the Bevy game engine.
//!
//! The 🎆 Hanabi particle system is a GPU-based particle system integrated with
//! the built-in Bevy renderer. It allows creating complex visual effects with
//! millions of particles simulated in real time, by leveraging the power of
//! compute shaders and offloading most of the work of particle simulating to
//! the GPU.
//!
//! The library supports the `wasm` target (WebAssembly) via the WebGPU renderer
//! only. Compute shaders are not available via the legacy WebGL2 renderer.
//! See Bevy's own [WebGL2 and WebGPU](https://github.com/bevyengine/bevy/tree/latest/examples#webgl2-and-webgpu)
//! section of the examples README for more information on how to run Wasm
//! builds with WebGPU.
//!
//! # 2D vs. 3D
//!
//! 🎆 Hanabi integrates both with the 2D and the 3D core pipelines of Bevy. The
//! 2D pipeline integration is controlled by the `2d` cargo feature, while the
//! 3D pipeline integration is controlled by the `3d` cargo feature. Both
//! features are enabled by default for convenience. As an optimization, users
//! can disable default features and re-enable only one of the two modes. At
//! least one of the `2d` or `3d` features must be enabled.
//!
//! ```toml
//! # Example: enable only 3D integration
//! bevy_hanabi = { version = "0.19", default-features = false, features = ["3d"] }
//! ```
//!
//! # Example
//!
//! Add the 🎆 Hanabi plugin to your app:
//!
//! ```no_run
//! # use bevy::prelude::*;
//! use bevy_hanabi::prelude::*;
//!
//! App::default()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(HanabiPlugin)
//!     .run();
//! ```
//!
//! Create an [`EffectAsset`] describing a visual effect:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_hanabi::prelude::*;
//! fn setup(mut effects: ResMut<Assets<EffectAsset>>) {
//!     // Define a color gradient from red to transparent black
//!     let mut gradient = bevy_hanabi::Gradient::new();
//!     gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
//!     gradient.add_key(1.0, Vec4::ZERO);
//!
//!     // Create a new expression module
//!     let mut module = Module::default();
//!
//!     // On spawn, randomly initialize the position of the particle
//!     // to be over the surface of a sphere of radius 2 units.
//!     let init_pos = SetPositionSphereModifier {
//!         center: module.lit(Vec3::ZERO),
//!         radius: module.lit(2.),
//!         dimension: ShapeDimension::Surface,
//!     };
//!
//!     // Also initialize a radial initial velocity to 6 units/sec
//!     // away from the (same) sphere center.
//!     let init_vel = SetVelocitySphereModifier {
//!         center: module.lit(Vec3::ZERO),
//!         speed: module.lit(6.),
//!     };
//!
//!     // Initialize the total lifetime of the particle, that is
//!     // the time for which it's simulated and rendered. This modifier
//!     // is almost always required, otherwise the particles will stay
//!     // alive forever, and new particles can't be spawned instead.
//!     let lifetime = module.lit(10.); // literal value "10.0"
//!     let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
//!
//!     // Every frame, add a gravity-like acceleration downward
//!     let accel = module.lit(Vec3::new(0., -3., 0.));
//!     let update_accel = AccelModifier::new(accel);
//!
//!     // Create the effect asset
//!     let effect = EffectAsset::new(
//!         // Maximum number of particles alive at a time
//!         1024,
//!         // Spawn at a rate of 5 particles per second
//!         SpawnerSettings::rate(5.0.into()),
//!         // Move the expression module into the asset
//!         module,
//!     )
//!     .with_name("MyEffect")
//!     .init(init_pos)
//!     .init(init_vel)
//!     .init(init_lifetime)
//!     .update(update_accel)
//!     // Render the particles with a color gradient over their
//!     // lifetime. This maps the gradient key 0 to the particle spawn
//!     // time, and the gradient key 1 to the particle death (10s).
//!     .render(ColorOverLifetimeModifier {
//!         gradient,
//!         blend: ColorBlendMode::Overwrite,
//!         mask: ColorBlendMask::RGBA,
//!     });
//!
//!     // Insert into the asset system
//!     let effect_asset = effects.add(effect);
//! }
//! ```
//!
//! Then add an instance of that effect to an entity by spawning a
//! [`ParticleEffect`] component referencing the asset.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_hanabi::prelude::*;
//! # fn spawn_effect(mut commands: Commands) {
//! #   let effect_asset = Handle::<EffectAsset>::default();
//! commands.spawn((
//!     ParticleEffect::new(effect_asset),
//!     Transform::from_translation(Vec3::Y),
//! ));
//! # }
//! ```
//!
//! # Workflow
//!
//! Authoring and using a particle effect follows this workflow:
//!
//! 1. Create an [`EffectAsset`] representing the definition of the particle
//!    effect. This asset is a proper Bevy [`Asset`], expected to be authored in
//!    advance, serialized, and shipped with your application. Creating an
//!    [`EffectAsset`] at runtime while the application is running is also
//!    supported, though. In any case however, the asset doesn't do anything by
//!    itself.
//! 2. At runtime, when the application is running, create an actual particle
//!    effect instance by spawning a [`ParticleEffect`] component. The component
//!    references the [`EffectAsset`] via its `handle` field. Multiple instances
//!    can reference the same asset at the same time, and some changes to the
//!    asset are reflected to its instances, although not all changes are
//!    supported. In general, avoid changing an [`EffectAsset`] while it's in
//!    use by one or more [`ParticleEffect`].
//! 3. If using properties, spawn an [`EffectProperties`] component on the same
//!    entity. Then update properties through that component at any time while
//!    the effect is active. This allows some moderate CPU-side control over the
//!    simulation and rendering of the effect, without having to destroy the
//!    effect and re-create a new one.
//! 4. If using textures, spawn an [`EffectMaterial`] component to define which
//!    texture is bound to which slot in the effect. An [`EffectAsset`] only
//!    defines "slots" of textures, not the actual assets bound to those slots.
//!    This way, you can reuse the same effect asset multiple times with
//!    different textures, like you'd do with a regular rendering mesh.
//! 5. For advanced VFX composed of multiple hierarchical effects, where two or
//!    more effects are connected to each other in a parent-child relationship,
//!    spawn an [`EffectParent`] on any child effect instance to specify its
//!    parent instance. See also the [`EmitSpawnEventModifier`].
//!
//! The [`EffectAsset`] is intended to be the serialized effect format, which
//! authors can save to disk and ship with their application. At this time
//! however serialization and deserialization is still a work in progress. In
//! particular, serialization and deserialization of all
//! [modifiers](crate::modifier) is currently not supported on `wasm` target.

use std::fmt::Write as _;

use bevy::{
    asset::AsAssetId,
    camera::visibility::VisibilityClass,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use rand::{RngExt as _, SeedableRng as _};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod asset;
pub mod attributes;
mod gradient;
pub mod graph;
pub mod modifier;
mod plugin;
pub mod properties;
mod aurora;
mod spawn;
mod time;

#[cfg(test)]
mod test_utils;

pub use asset::{
    AlphaMode, EffectAsset, EffectAssetDeserializer, EffectAssetLoader,
    EffectAssetSerializer, EffectParent, MotionIntegration, SimulationCondition,
};
pub use attributes::*;
pub use gradient::{Gradient, GradientKey};
pub use graph::*;
pub use modifier::*;
pub use plugin::{EffectSystems, HanabiPlugin};
pub use properties::*;
pub use aurora::LayoutFlags;
pub use spawn::{tick_spawners, CpuValue, EffectSpawner, Random, SpawnerSettings};
pub use time::{EffectSimulation, EffectSimulationTime};

/// The members of a Slang struct laid out at the given byte offsets, in Slang's natural
/// (scalar) layout: each gap, and the tail up to `stride`, an explicit `uint` pad, so the GPU
/// reads every field where the CPU layout put it. Entries are `(offset, name, type, size)`.
pub(crate) fn slang_struct_members<'a>(
    entries: impl IntoIterator<Item = (u32, &'a str, String, u32)>,
    stride: u32,
) -> String {
    let mut code = String::new();
    let mut at = 0u32;
    let mut pads = 0u32;
    let mut pad = |code: &mut String, from: u32, to: u32| {
        debug_assert!((to - from) % 4 == 0);
        for _ in 0..(to - from) / 4 {
            code.push_str(&format!("    uint _pad{pads};\n"));
            pads += 1;
        }
    };
    for (offset, name, ty, size) in entries {
        if offset > at {
            pad(&mut code, at, offset);
        }
        code.push_str(&format!("    {ty} {name};\n"));
        at = offset + size;
    }
    if stride > at {
        pad(&mut code, at, stride);
    }
    code
}

#[allow(missing_docs)]
pub mod prelude {
    #[doc(hidden)]
    pub use crate::*;
}


/// Extension trait to convert an object to WGSL code.
///
/// This is mainly used for floating-point constants. This is required because
/// WGSL doesn't support a floating point constant without a decimal separator
/// (_e.g._ `0.` instead of `0`), which would be what a regular string
/// formatting function like [`format!()`] would produce, but which is
/// interpreted as an integral type by WGSL instead.
///
/// # Example
///
/// ```
/// # use bevy_hanabi::ToSlangString;
/// let x = 2.0_f32;
/// assert_eq!("let x = 2.;", format!("let x = {};", x.to_slang_string()));
/// ```
///
/// # Remark
///
/// This trait is soft-deprecated. It serves the same purpose as
/// [`EvalContext::eval()`], however it lacks any context for the evaluation of
/// an expression producing WGSL code, so its use is limited. It's still useful
/// for constant (literal) expressions, which do not require any context to
/// evaluate.
///
/// [`format!()`]: std::format
/// [`EvalContext::eval()`]: crate::graph::expr::EvalContext::eval
pub trait ToSlangString {
    /// Convert an object to a string representing its WGSL code.
    fn to_slang_string(&self) -> String;
}

impl ToSlangString for f32 {
    fn to_slang_string(&self) -> String {
        let s = format!("{self:.6}");
        s.trim_end_matches('0').to_string()
    }
}

impl ToSlangString for f64 {
    fn to_slang_string(&self) -> String {
        let s = format!("{self:.15}");
        s.trim_end_matches('0').to_string()
    }
}

impl ToSlangString for Vec2 {
    fn to_slang_string(&self) -> String {
        format!(
            "float2({0},{1})",
            self.x.to_slang_string(),
            self.y.to_slang_string()
        )
    }
}

impl ToSlangString for Vec3 {
    fn to_slang_string(&self) -> String {
        format!(
            "float3({0},{1},{2})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string()
        )
    }
}

impl ToSlangString for Vec4 {
    fn to_slang_string(&self) -> String {
        format!(
            "float4({0},{1},{2},{3})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string(),
            self.w.to_slang_string()
        )
    }
}

impl ToSlangString for bool {
    fn to_slang_string(&self) -> String {
        if *self {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }
}

impl ToSlangString for BVec2 {
    fn to_slang_string(&self) -> String {
        format!(
            "bool2({0},{1})",
            self.x.to_slang_string(),
            self.y.to_slang_string()
        )
    }
}

impl ToSlangString for BVec3 {
    fn to_slang_string(&self) -> String {
        format!(
            "bool3({0},{1},{2})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string()
        )
    }
}

impl ToSlangString for BVec4 {
    fn to_slang_string(&self) -> String {
        format!(
            "bool4({0},{1},{2},{3})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string(),
            self.w.to_slang_string()
        )
    }
}

impl ToSlangString for i32 {
    fn to_slang_string(&self) -> String {
        format!("{}", self)
    }
}

impl ToSlangString for IVec2 {
    fn to_slang_string(&self) -> String {
        format!(
            "int2({0},{1})",
            self.x.to_slang_string(),
            self.y.to_slang_string()
        )
    }
}

impl ToSlangString for IVec3 {
    fn to_slang_string(&self) -> String {
        format!(
            "int3({0},{1},{2})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string()
        )
    }
}

impl ToSlangString for IVec4 {
    fn to_slang_string(&self) -> String {
        format!(
            "int4({0},{1},{2},{3})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string(),
            self.w.to_slang_string()
        )
    }
}

impl ToSlangString for u32 {
    fn to_slang_string(&self) -> String {
        format!("{}u", self)
    }
}

impl ToSlangString for UVec2 {
    fn to_slang_string(&self) -> String {
        format!(
            "uint2({0},{1})",
            self.x.to_slang_string(),
            self.y.to_slang_string()
        )
    }
}

impl ToSlangString for UVec3 {
    fn to_slang_string(&self) -> String {
        format!(
            "uint3({0},{1},{2})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string()
        )
    }
}

impl ToSlangString for UVec4 {
    fn to_slang_string(&self) -> String {
        format!(
            "uint4({0},{1},{2},{3})",
            self.x.to_slang_string(),
            self.y.to_slang_string(),
            self.z.to_slang_string(),
            self.w.to_slang_string()
        )
    }
}

impl ToSlangString for CpuValue<f32> {
    fn to_slang_string(&self) -> String {
        match self {
            Self::Single(x) => x.to_slang_string(),
            Self::Uniform((a, b)) => format!(
                "(frand() * ({1} - {0}) + {0})",
                a.to_slang_string(),
                b.to_slang_string(),
            ),
        }
    }
}

impl ToSlangString for CpuValue<Vec2> {
    fn to_slang_string(&self) -> String {
        match self {
            Self::Single(v) => v.to_slang_string(),
            Self::Uniform((a, b)) => format!(
                "(frand2() * ({1} - {0}) + {0})",
                a.to_slang_string(),
                b.to_slang_string(),
            ),
        }
    }
}

impl ToSlangString for CpuValue<Vec3> {
    fn to_slang_string(&self) -> String {
        match self {
            Self::Single(v) => v.to_slang_string(),
            Self::Uniform((a, b)) => format!(
                "(frand3() * ({1} - {0}) + {0})",
                a.to_slang_string(),
                b.to_slang_string(),
            ),
        }
    }
}

impl ToSlangString for CpuValue<Vec4> {
    fn to_slang_string(&self) -> String {
        match self {
            Self::Single(v) => v.to_slang_string(),
            Self::Uniform((a, b)) => format!(
                "(frand4() * ({1} - {0}) + {0})",
                a.to_slang_string(),
                b.to_slang_string(),
            ),
        }
    }
}

/// Simulation space for the particles of an effect.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SimulationSpace {
    /// Particles are simulated in global space.
    ///
    /// The global space is the Bevy world space. Particles simulated in global
    /// space are "detached" from the emitter when they spawn, and not
    /// influenced anymore by the emitter's [`GlobalTransform`] after spawning.
    /// The particle's [`Attribute::POSITION`] is the world space position
    /// of the particle.
    ///
    /// This is the default.
    #[default]
    Global,

    /// Particles are simulated in local effect space.
    ///
    /// The local space is the space associated with the [`GlobalTransform`] of
    /// the [`ParticleEffect`] component being simulated. Particles
    /// simulated in local effect space are "attached" to the effect, and
    /// will be affected by its [`GlobalTransform`]. The particle's
    /// [`Attribute::POSITION`] is the position of the particle relative to
    /// the effect's [`GlobalTransform`].
    Local,
}

impl SimulationSpace {
    /// Evaluate the simulation space expression.
    ///
    /// - Init: [`SimulationSpace::Global`] converts emitter space to simulation
    ///   space; [`SimulationSpace::Local`] is a no-op.
    /// - Update: no-op. Particles are already in simulation space.
    /// - In the render context, this expression transforms the particle's
    ///   position from simulation space to view space.
    pub fn eval(&self, context: &dyn EvalContext) -> Result<String, ExprError> {
        match context.modifier_context() {
            ModifierContext::Init => match *self {
                SimulationSpace::Global => {
                    if !context.particle_layout().contains(Attribute::POSITION) {
                        return Err(ExprError::GraphEvalError(format!("Global-space simulation requires that the particles have a {} attribute.", Attribute::POSITION.name())));
                    }
                    let pos = Attribute::POSITION.name();
                    let mut code = format!(
                        "particle.{0} = mul(transform, float4(particle.{0}, 1.0)).xyz;\n",
                        pos
                    );
                    if context.particle_layout().contains(Attribute::VELOCITY) {
                        let vel = Attribute::VELOCITY.name();
                        code.push_str(&format!(
                            "particle.{0} = mul(transform, float4(particle.{0}, 0.0)).xyz;\n",
                            vel
                        ));
                    }
                    Ok(code)
                }
                SimulationSpace::Local => Ok("".to_string()),
            },
            ModifierContext::Update => Ok(String::new()),
            ModifierContext::Render => Ok(match *self {
                // TODO: cast vec3 -> vec4 auomatically
                SimulationSpace::Global => "float4(local_position, 1.0)",
                // TODO: transform_world_to_view(...)
                SimulationSpace::Local => "mul(transform, float4(local_position, 1.0))",
            }
            .to_string()),
            _ => Err(ExprError::GraphEvalError(
                "Invalid modifier context value.".to_string(),
            )),
        }
    }
}

/// Value a user wants to assign to a property with
/// [`EffectProperties::set()`] before the instance had a chance
/// to inspect its underlying asset and check the asset's defined properties.
///
/// A property with this name might not exist, in which case the value will be
/// discarded silently when the instance is initialized from its asset.
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct PropertyValue {
    /// Name of the property the value should be assigned to.
    name: String,

    /// The property value to assign, instead of the default value of the
    /// property.
    value: Value,
}

impl From<PropertyInstance> for PropertyValue {
    fn from(prop: PropertyInstance) -> Self {
        Self {
            name: prop.def.name().to_string(),
            value: prop.value,
        }
    }
}

impl From<&PropertyInstance> for PropertyValue {
    fn from(prop: &PropertyInstance) -> Self {
        Self {
            name: prop.def.name().to_string(),
            value: prop.value,
        }
    }
}

/// Marker component to tag the "main" camera.
///
/// Various features in Hanabi need a single camera to work with. However, Bevy
/// doesn't have a concept of a "main" camera. This component solves this and
/// marks a camera as the "main" one that Hanabi effects will use for camera
/// related features, like sorting by camera distance (for semi-transparent
/// particles), reading the depth buffer (_e.g._ for collision), or checking the
/// camera frustum bounds (_e.g._ for frustum culling).
///
/// The component must be added to at most one entity
/// having a [`Camera`] component. If absent, camera-related features will not
/// work (but all other features continue to work). If no camera-related feature
/// is used, adding this component is not required. Generally you want to add it
/// to the camera that renders the "main view" of your game or application, the
/// one that defines the final rendered view.
///
/// Dynamically switching camera is supported, but note that only one camera per
/// frame can be tagged (the unique camera tagged at the time of the render
/// world extraction phase is the camera taken into account). Tagging multiple
/// camera will result in a system error and break Hanabi rendering.
#[derive(Default, Clone, Copy, Component)]
pub struct HanabiMainCamera;

/// The [`VisibilityClass`] used for all particle effects.
#[derive(Default, Clone, Copy, Component)]
pub struct EffectVisibilityClass;

/// Particle-based visual effect instance.
///
/// The particle effect component represents a single instance of a visual
/// effect. The visual effect itself is described by a handle to an
/// [`EffectAsset`].
///
/// This instance is associated to an [`Entity`], inheriting
/// its [`GlobalTransform`] as the origin frame for its particle spawning.
///
/// # Content
///
/// The values in this component, with the exception of the handle to the
/// underlying asset itself ([`ParticleEffect::handle`]), are optional
/// per-instance overrides taking precedence over the fallback value set in the
/// [`EffectAsset`].
///
/// Note that the effect graph itself including all its modifiers cannot be
/// overridden per instance. For minor variations use different assets. If you
/// need too many variants try to use a property instead.
///
/// # Component dependencies
///
/// ## Mandatory components
///
/// The [`ParticleEffect`] component requires, in the sense of ECS, some other
/// components for the particle effect to work. Those mandatory components are
/// automatically added by Bevy if not otherwise provided during spawning.
///
/// - A [`CompiledParticleEffect`] component, which contains some precomputed
///   data and allocated resources, in particular GPU resources.
/// - A [`Visibility`] component (and the components it requires in turn) to
///   make use of the Bevy visibility system and optimize rendering of the
///   effects. This influences simulation when using
///   [`SimulationCondition::WhenVisible`].
/// - A [`Transform`] component to define the position of the particle emitter.
///
/// ## Optional components
///
/// - The [`EffectMaterial`] defines the "material" of the particle effect,
///   which contains for example the textures used by the particle system. This
///   component is optional, because not all effects make use of a material.
/// - The [`EffectParent`] defines the parent effect of this effect. This is
///   used for hierarchical effect construction, and to use GPU spawn events to
///   make the parent effect trigger spawning particles in this effect.
/// - The [`EffectProperties`] defines the runtime values of the properties of
///   the effect. If the effect doesn't use properties, this component is not
///   used.
///
/// # Change detection
///
/// The [`CompiledParticleEffect`] component located on the same [`Entity`] as
/// the current [`ParticleEffect`] component is automatically updated when a
/// change occurs to this component. This separation is designed to leverage the
/// change detection system of Bevy, in order to ensure that any manual change
/// by the user is performed on this component and any automated change is
/// performed on the [`CompiledParticleEffect`] component. This allows
/// efficiently detecting when a user change requires an update to the compiled
/// particle effect, while ensuring conversely that internal mutations do not
/// invalidate the compiled effect and do not trigger a costly shader rebuild
/// for example.
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
#[require(CompiledParticleEffect, Transform, Visibility, VisibilityClass)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<EffectVisibilityClass>)]
pub struct ParticleEffect {
    /// Handle of the effect to instantiate.
    pub handle: Handle<EffectAsset>,
    /// Optional per-instance PRNG seed.
    ///
    /// Set this value to `Some(seed)` to override the default value set in
    /// [`EffectAsset::prng_seed`] for this effect instance. By default this is
    /// `None`, and the instance uses the same PRNG seed as its [`EffectAsset`].
    pub prng_seed: Option<u32>,
}

impl ParticleEffect {
    /// Create a new particle effect instance from an existing asset.
    pub fn new(handle: Handle<EffectAsset>) -> Self {
        Self {
            handle,
            prng_seed: None,
        }
    }
}

impl AsAssetId for ParticleEffect {
    type Asset = EffectAsset;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.handle.id()
    }
}

/// Material for an effect instance.
///
/// A material component contains the render resources (textures) for a single
/// effect instance. Those textures are automatically bound during rendering to
/// the slots defined with [`Module::add_texture_slot()`]. Using this, multiple
/// effect instances sharing a same source [`EffectAsset`] can be instantiated
/// and rendered with different sets of textures, without changing the asset.
///
/// The [`EffectMaterial`] component needs to be spawned on the same entity as
/// the [`ParticleEffect`] component representing the effect instance.
#[derive(Debug, Default, Clone, Component)]
pub struct EffectMaterial {
    /// List of texture images to use to render the effect instance.
    ///
    /// The images are ordered by [slot index] into the corresponding
    /// [`TextureLayout`].
    ///
    /// [slot index]: crate::TextureLayout::get_slot_by_name
    pub images: Vec<Handle<Image>>,
}

/// Dimension of a texture for a material slot.
///
/// This defines the type of WGSL texture read or sampler used, and what format
/// is expected for the [`Image`] bound to that slot. Image binding is done via
/// the [`EffectMaterial`] component.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Default)]
pub enum SlotDimension {
    /// A one dimensional texture. `texture_1d` in WGSL.
    #[serde(rename = "1d")]
    D1,

    /// A two dimensional texture. `texture_2d` in WGSL.
    #[serde(rename = "2d")]
    #[default]
    D2,

    /// A two dimensional array texture. `texture_2d_array` in WGSL.
    #[serde(rename = "2d-array")]
    D2Array,

    /// A cubemap texture. `texture_cube` in WGSL.
    #[serde(rename = "cube")]
    Cube,

    /// A cubemap array texture. `texture_cube_array` in WGSL.
    #[serde(rename = "cube-array")]
    CubeArray,

    /// A three dimensional texture. `texture_3d` in WGSL.
    #[serde(rename = "3d")]
    D3,

    /// A two dimensional depth texture. `texture_depth_2d`.
    #[serde(rename = "depth-2d")]
    DepthD2,

    /// A two dimensional array texture. `texture_depth_2d_array` in WGSL.
    #[serde(rename = "depth-2d-array")]
    DepthD2Array,

    /// A cubemap texture. `texture_depth_cube` in WGSL.
    #[serde(rename = "depth-cube")]
    DepthCube,

    /// A cubemap array texture. `texture_depth_cube_array` in WGSL.
    #[serde(rename = "depth-cube-array")]
    DepthCubeArray,
}

impl SlotDimension {
    /// Check if this slot dimension uses an array texture.
    ///
    /// Array textures require an addition `array_index` when loading a value or
    /// sampling the texture. See [`TextureSampleExpr`] and [`TextureLoadExpr`].
    pub fn is_array(&self) -> bool {
        matches!(
            *self,
            SlotDimension::D2Array
                | SlotDimension::CubeArray
                | SlotDimension::DepthD2Array
                | SlotDimension::DepthCubeArray
        )
    }

    /// Check if this slot dimension uses a depth texture.
    ///
    /// Depth textures use a comparison sampler, and always return a scalar
    /// `f32` value. Non-depth textures conversely return a `vec4<f32>`, and use
    /// an interpolating sampler (and optional mip-map).
    pub fn is_depth(&self) -> bool {
        matches!(
            *self,
            SlotDimension::DepthD2
                | SlotDimension::DepthD2Array
                | SlotDimension::DepthCube
                | SlotDimension::DepthCubeArray
        )
    }

    /// Check if this slot dimension uses a cube texture.
    ///
    /// Cube textures have some usage restrictions. In particular, you cannot
    /// use them with a [`TextureLoadExpr`]. They always need a multiple of 6
    /// layers (and exactly 6 if not an array).
    pub fn is_cube(&self) -> bool {
        matches!(
            *self,
            SlotDimension::Cube
                | SlotDimension::CubeArray
                | SlotDimension::DepthCube
                | SlotDimension::DepthCubeArray
        )
    }
}

/// Texture slot of a [`Module`].
///
/// A texture slot defines a named bind point where a texture can be attached
/// and read/sampled by an effect during rendering. A slot also has an implicit
/// unique index corresponding to its position in the [`TextureLayout::layout`]
/// array of the effect.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct TextureSlot {
    /// Unique slot name.
    pub name: String,
    /// Texture dimension the slot uses. This determines the sampler of the slot
    /// in the WGSL shader.
    pub dimension: SlotDimension,
}

/// Texture layout.
///
/// Defines the list of texture slots for an effect.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct TextureLayout {
    /// The list of slots.
    ///
    /// The slots index corresponds to the index inside this array. Each image
    /// in [`EffectMaterial::images`] maps by index to a unique slot in this
    /// array.
    pub layout: Vec<TextureSlot>,
}

impl TextureLayout {
    /// Find a texture slot by name, and return its index.
    ///
    /// The index uniquely identify the slot (like its name), and indexes into
    /// the [`EffectMaterial::images`] array to determine which texture is bound
    /// to it.
    pub fn get_slot_by_name(&self, name: &str) -> Option<usize> {
        self.layout.iter().position(|slot| slot.name == name)
    }
}

/// Optional mesh override for a particle effect.
///
/// Add this component to the same entity as a [`ParticleEffect`] to override the mesh each
/// particle traces as (an [`AuroraMesh`]; the default is an analytic sphere).
///
/// [`AuroraMesh`]: bevy_aurora::mesh::AuroraMesh
#[derive(Debug, Default, Clone, PartialEq, Component)]
pub struct EffectMesh(pub Handle<bevy_aurora::mesh::AuroraMesh>);

/// Slang source of an effect's kernels.
///
/// Generated from an [`EffectAsset`] by applying all its modifiers: two self-contained
/// modules (aurora compiles each with slangc), the init kernel (`init`) and the update kernel
/// (`update`, which also runs the render modifiers and writes the particles' instance rows).
/// Exposed for debugging and inspection (editor).
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectShaderSources {
    /// Slang source of the init kernel.
    pub init_shader_source: String,
    /// Slang source of the update kernel.
    pub update_shader_source: String,
    /// Effect flags the sources were generated with.
    pub layout_flags: LayoutFlags,
}

/// Error resulting from the generating of the WGSL shader code of an
/// [`EffectAsset`].
#[derive(Debug, Error)]
pub enum ShaderGenerateError {
    /// Error related to an [`Expr`].
    #[error("Expression error: {0}")]
    Expr(ExprError),

    /// Shader validation error.
    #[error("Validation error: {0}")]
    Validate(String),
}

impl EffectShaderSources {
    /// Generate the effect shader WGSL source code.
    ///
    /// This takes a base asset effect and generate the WGSL code for the
    /// various shaders (init/update/render).
    pub fn generate(
        asset: &EffectAsset,
        // Note: ideally the below fields are folded into the asset, but currently the parent
        // relationship and GPU event one are not encoded in assets.
        parent_layout: Option<&ParticleLayout>,
        num_event_bindings: u32,
    ) -> Result<EffectShaderSources, ShaderGenerateError> {
        trace!(
            "Generating shader sources for asset '{}' with {} event bindings",
            asset.name,
            num_event_bindings
        );

        let particle_layout = asset.particle_layout();

        // The particle layout cannot be empty currently because we always emit some
        // Particle{} struct and it needs at least one field. There's probably no use
        // case for an empty layout anyway.
        if particle_layout.size() == 0 {
            return Err(ShaderGenerateError::Validate(format!(
                "Asset {} has invalid empty particle layout.",
                asset.name
            )));
        }
        if let Some(parent_layout) = parent_layout {
            if parent_layout.size() == 0 {
                return Err(ShaderGenerateError::Validate(format!(
                    "Effect using asset {} has invalid empty parent particle layout.",
                    asset.name
                )));
            }
        }

        // Currently the POSITION attribute is mandatory, as it's always used by the
        // render shader.
        if !particle_layout.contains(Attribute::POSITION) {
            return Err(ShaderGenerateError::Validate(format!(
                "The particle layout of asset '{}' is missing the '{}' attribute. Add a modifier using that attribute, for example the SetAttributeModifier.",
                asset.name, Attribute::POSITION.name().to_ascii_uppercase()
            )));
        }

        // Currently ribbon rendering requires AGE, so warn if it's missing because
        // everything will break with some weird error aboud bind groups or whatnot.
        if particle_layout.contains(Attribute::RIBBON_ID)
            && !particle_layout.contains(Attribute::AGE)
        {
            return Err(ShaderGenerateError::Validate(format!(
                "The particle layout of asset '{}' uses ribbons (has the '{}' attribute), but is missing the '{}' attribute, which is mandatory for ribbons. Add a modifier using that attribute, for example the SetAttributeModifier.",
                asset.name, Attribute::RIBBON_ID.name().to_ascii_uppercase(), Attribute::AGE.name().to_ascii_uppercase()
            )));
        }

        // Generate the WGSL code declaring all the attributes inside the Particle
        // struct.
        let attributes_code = particle_layout.generate_code();
        let parent_particle_code = parent_layout
            .map(|layout| {
                format!(
                    "struct ParentParticle {{\n{}}};",
                    layout.generate_code()
                )
            })
            .unwrap_or_default();

        // For the renderer, assign all its inputs to the values of the attributes
        // present, or a default value.
        let mut inputs_code = String::new();
        // All required attributes, except the size/color which are variadic
        let required_attributes =
            HashSet::from_iter([Attribute::AXIS_X, Attribute::AXIS_Y, Attribute::AXIS_Z]);
        let mut present_attributes = HashSet::new();
        let mut has_size = false;
        let mut has_color = false;
        for attr_layout in particle_layout.attributes() {
            let attr = attr_layout.attribute;
            if attr == Attribute::SIZE {
                if !has_size {
                    inputs_code += &format!(
                        "var size = float3(particle.{0}, particle.{0}, particle.{0});\n",
                        Attribute::SIZE.name()
                    );
                    has_size = true;
                    present_attributes.insert(attr);
                } else {
                    warn!("Attribute SIZE conflicts with another size attribute; ignored.");
                }
            } else if attr == Attribute::SIZE2 {
                if !has_size {
                    inputs_code += &format!(
                        "var size = float3(particle.{0}, 1.0);\n",
                        Attribute::SIZE2.name()
                    );
                    has_size = true;
                    present_attributes.insert(attr);
                } else {
                    warn!("Attribute SIZE2 conflicts with another size attribute; ignored.");
                }
            } else if attr == Attribute::SIZE3 {
                if !has_size {
                    inputs_code += &format!("var size = particle.{0};\n", Attribute::SIZE3.name());
                    has_size = true;
                    present_attributes.insert(attr);
                } else {
                    warn!("Attribute SIZE3 conflicts with another size attribute; ignored.");
                }
            } else if attr == Attribute::HDR_COLOR {
                if !has_color {
                    inputs_code +=
                        &format!("var color = particle.{};\n", Attribute::HDR_COLOR.name());
                    has_color = true;
                    present_attributes.insert(attr);
                } else {
                    warn!("Attribute HDR_COLOR conflicts with another color attribute; ignored.");
                }
            } else if attr == Attribute::COLOR {
                if !has_color {
                    inputs_code += &format!(
                        "var color = unpack4x8unorm(particle.{0});\n",
                        Attribute::COLOR.name()
                    );
                    has_color = true;
                    present_attributes.insert(attr);
                } else {
                    warn!("Attribute COLOR conflicts with another color attribute; ignored.");
                }
            } else {
                inputs_code += &format!("var {0} = particle.{0};\n", attr.name());
                present_attributes.insert(attr);
            }
        }
        // For all attributes required by the render shader, if they're not explicitly
        // stored in the particle layout, define a variable with their default value.
        if !has_size {
            inputs_code += &format!(
                "var size = {0};\n",
                Attribute::SIZE3.default_value().to_slang_string() // TODO - or SIZE?
            );
        }
        if !has_color {
            inputs_code += &format!(
                "var color = {};\n",
                Attribute::HDR_COLOR.default_value().to_slang_string() // TODO - or COLOR?
            );
        }
        for &attr in required_attributes.difference(&present_attributes) {
            inputs_code += &format!(
                "var {} = {};\n",
                attr.name(),
                attr.default_value().to_slang_string()
            );
        }

        // Generate the shader code defining the per-effect properties, if any
        let property_layout = asset.property_layout();
        let properties_code = property_layout
            .generate_property_struct_code()
            .unwrap_or_else(|| "struct Properties {\n    uint _unused;\n};\n".to_string());
        let texture_layout = asset.texture_layout();

        if num_event_bindings > 0 {
            warn!(
                "Effect '{}' has {} child effect(s): GPU spawn events are not on aurora yet.",
                asset.name, num_event_bindings
            );
        }

        // Start from the base module containing the expressions actually serialized in
        // the asset. We will add the ones created on-the-fly by applying the
        // modifiers to the contexts.
        let mut module = asset.module().clone();

        let mut layout_flags = LayoutFlags::NONE;
        if asset.simulation_space == SimulationSpace::Local {
            layout_flags |= LayoutFlags::LOCAL_SPACE_SIMULATION;
        }
        match &asset.alpha_mode {
            AlphaMode::Mask(_) => layout_flags.insert(LayoutFlags::USE_ALPHA_MASK),
            AlphaMode::Opaque => layout_flags.insert(LayoutFlags::OPAQUE),
            _ => layout_flags.remove(LayoutFlags::USE_ALPHA_MASK | LayoutFlags::OPAQUE),
        }
        if particle_layout.contains(Attribute::RIBBON_ID) {
            layout_flags |= LayoutFlags::RIBBONS;
        }
        if parent_layout.is_some() {
            layout_flags |= LayoutFlags::READ_PARENT_PARTICLE;
        }

        // Generate the shader code for the initializing shader
        let (init_code, init_extra, init_sim_space_transform_code, consume_gpu_spawn_events) = {
            // Apply all the init modifiers
            let mut init_context = ShaderWriter::new(
                ModifierContext::Init,
                &property_layout,
                &particle_layout,
                &texture_layout,
            );
            for m in asset.init_modifiers() {
                if let Err(err) = m.apply(&mut module, &mut init_context) {
                    error!(
                        "Failed to compile effect '{}', error in init context: {}",
                        asset.name, err
                    );
                    return Err(ShaderGenerateError::Expr(err));
                }
            }

            let sim_space_transform_code =
                asset.simulation_space.eval(&init_context).map_err(|err| {
                    error!("Failed to compile effect's simulation space: {}", err);
                    ShaderGenerateError::Expr(err)
                })?;

            // Effect uses GPU spawn events if it produces them, or if it has a parent and
            // consumes them from that parent.
            let consume_gpu_spawn_events =
                init_context.emits_gpu_spawn_events().unwrap_or(false) || parent_layout.is_some();

            (
                init_context.main_code,
                init_context.extra_code,
                sim_space_transform_code,
                consume_gpu_spawn_events,
            )
        };

        let local_space = asset.simulation_space == SimulationSpace::Local;
        let common = effect_common_source(
            local_space,
            &attributes_code,
            &parent_particle_code,
            &properties_code,
        );
        let init_shader_source = common.clone()
            + &PARTICLES_INIT_SHADER_TEMPLATE
                .replace("{{INIT_CODE}}", &init_code)
                .replace("{{INIT_EXTRA}}", &init_extra)
                .replace(
                    "{{SIMULATION_SPACE_TRANSFORM_PARTICLE}}",
                    &init_sim_space_transform_code,
                );
        trace!(
            "Configured init shader for '{}':\n{}",
            asset.name,
            init_shader_source
        );

        // Generate the shader code for the update shader
        let (mut update_code, update_extra, emit_gpu_spawn_events) = {
            let mut update_context = ShaderWriter::new(
                ModifierContext::Update,
                &property_layout,
                &particle_layout,
                &texture_layout,
            );
            for m in asset.update_modifiers() {
                if let Err(err) = m.apply(&mut module, &mut update_context) {
                    error!(
                        "Failed to compile effect '{}', error in update context: {}",
                        asset.name, err
                    );
                    return Err(ShaderGenerateError::Expr(err));
                }
            }

            let emit_gpu_spawn_events = update_context.emits_gpu_spawn_events().unwrap_or(false);

            (
                update_context.main_code,
                update_context.extra_code,
                emit_gpu_spawn_events,
            )
        };

        if consume_gpu_spawn_events {
            layout_flags |= LayoutFlags::CONSUME_GPU_SPAWN_EVENTS;
        }
        if emit_gpu_spawn_events {
            layout_flags |= LayoutFlags::EMIT_GPU_SPAWN_EVENTS;
        }

        // Insert Euler motion integration if needed.
        let has_position = present_attributes.contains(&Attribute::POSITION);
        let has_velocity = present_attributes.contains(&Attribute::VELOCITY);
        if asset.motion_integration != MotionIntegration::None {
            if has_position && has_velocity {
                // Note the prepended "\n" to prevent appending to a comment line.
                let code = format!(
                    "\nparticle.{0} += particle.{1} * sim_params.delta_time;\n",
                    Attribute::POSITION.name(),
                    Attribute::VELOCITY.name()
                );
                if asset.motion_integration == MotionIntegration::PreUpdate {
                    update_code.insert_str(0, &code);
                } else {
                    update_code += &code;
                }
            } else {
                warn!(
                        "Asset '{}' specifies motion integration but is missing {}. Particles won't move unless the POSITION attribute is explicitly assigned. Set MotionIntegration::None to remove this warning.",
                        asset.name,
                        if has_position {
                            "Attribute::VELOCITY"
                        } else {
                            "Attribute::POSITION"
                        }
                    )
            }
        }

        // Generate the shader code for the render shader
        let (
            vertex_code,
            fragment_code,
            render_extra,
            alpha_cutoff_code,
            flipbook_scale_code,
            flipbook_row_count_code,
        ) = {
            let texture_layout = module.texture_layout();
            let mut render_context =
                RenderContext::new(&property_layout, &particle_layout, &texture_layout);
            for m in asset.render_modifiers() {
                m.apply_render(&mut module, &mut render_context)
                    .map_err(ShaderGenerateError::Expr)?;
            }

            if render_context.needs_uv {
                layout_flags |= LayoutFlags::NEEDS_UV;
            }
            if render_context.needs_normal {
                layout_flags |= LayoutFlags::NEEDS_NORMAL;
            }
            if render_context.needs_particle_fragment {
                layout_flags |= LayoutFlags::NEEDS_PARTICLE_FRAGMENT;
            }

            let alpha_cutoff_code = if let AlphaMode::Mask(cutoff) = &asset.alpha_mode {
                render_context.eval(&module, *cutoff).unwrap_or_else(|err| {
                    error!(
                        "Failed to evaluate the expression for AlphaMode::Mask, error: {}",
                        err
                    );

                    // In Debug, show everything to help diagnosing
                    #[cfg(debug_assertions)]
                    return 1_f32.to_slang_string();

                    // In Release, hide everything with an error
                    #[cfg(not(debug_assertions))]
                    return 0_f32.to_slang_string();
                })
            } else {
                // WESL parses disabled branches before conditional translation.
                // Keep the unused initializer syntactically valid.
                "0.0".to_string()
            };

            let (flipbook_scale_code, flipbook_row_count_code) = if let Some(grid_size) =
                render_context.sprite_grid_size
            {
                layout_flags |= LayoutFlags::FLIPBOOK;
                // Note: row_count needs to be i32, not u32, because of sprite_index
                let flipbook_row_count_code = (grid_size.x as i32).to_slang_string();
                let flipbook_scale_code =
                    Vec2::new(1.0 / grid_size.x as f32, 1.0 / grid_size.y as f32).to_slang_string();
                (flipbook_scale_code, flipbook_row_count_code)
            } else {
                ("float2(1.0)".to_string(), "1i".to_string())
            };

            (
                render_context.vertex_code,
                render_context.fragment_code,
                render_context.render_extra,
                alpha_cutoff_code,
                flipbook_scale_code,
                flipbook_row_count_code,
            )
        };

        // Configure aging code
        let has_age = present_attributes.contains(&Attribute::AGE);
        let has_lifetime = present_attributes.contains(&Attribute::LIFETIME);
        let mut age_code = String::new();
        if has_age {
            if has_lifetime {
                age_code += &format!(
                    "\n    let was_alive = particle.{0} < particle.{1};",
                    Attribute::AGE.name(),
                    Attribute::LIFETIME.name()
                );
            }

            age_code += &format!(
                "\n    particle.{0} = particle.{0} + sim_params.delta_time;",
                Attribute::AGE.name()
            );

            if has_lifetime {
                age_code += &format!(
                    "\n    var is_alive = particle.{0} < particle.{1};",
                    Attribute::AGE.name(),
                    Attribute::LIFETIME.name()
                );
            }
        } else {
            // Since we're using a dead index buffer, all particles that make it to the
            // update compute shader are guaranteed to be alive (we never
            // simulate dead particles).
            age_code = "\n    let was_alive = true;\n    var is_alive = true;".to_string();
        }

        // Configure reaping code
        let reap_code = if has_age && has_lifetime {
            format!(
                "is_alive = is_alive && (particle.{0} < particle.{1});",
                Attribute::AGE.name(),
                Attribute::LIFETIME.name()
            )
        } else {
            "".to_string()
        };

        // Assign attributes individually instead of using struct
        // assignment. Otherwise we might race on `PREV` and `NEXT`
        // attributes, which might be updated behind our back when adjacent
        // particles die.
        let mut writeback_code = "".to_owned();
        for attribute in present_attributes
            .iter()
            .filter(|attribute| **attribute != Attribute::PREV && **attribute != Attribute::NEXT)
        {
            writeln!(
                &mut writeback_code,
                "    push.particles[slot].{0} = particle.{0};",
                attribute.name()
            )
            .unwrap();
        }

        // Configure the update shader template, and make sure a corresponding shader
        // asset exists
        if !fragment_code.trim().is_empty() {
            warn!(
                "Effect '{}' has fragment-stage render modifiers (textures, alpha), which aurora has not: ignored.",
                asset.name
            );
        }
        let _ = (alpha_cutoff_code, flipbook_scale_code, flipbook_row_count_code);
        let update_shader_source = common
            + &PARTICLES_UPDATE_SHADER_TEMPLATE
                .replace("{{AGE_CODE}}", &age_code)
                .replace("{{REAP_CODE}}", &reap_code)
                .replace("{{UPDATE_CODE}}", &update_code)
                .replace("{{WRITEBACK_CODE}}", &writeback_code)
                .replace("{{UPDATE_EXTRA}}", &update_extra)
                .replace("{{INPUTS}}", &inputs_code)
                .replace("{{VERTEX_MODIFIERS}}", &vertex_code)
                .replace("{{RENDER_EXTRA}}", &render_extra);
        trace!(
            "Configured update shader for '{}':\n{}",
            asset.name,
            update_shader_source
        );

        Ok(EffectShaderSources {
            init_shader_source,
            update_shader_source,
            layout_flags,
        })
    }
}

/// Compiled data for a [`ParticleEffect`].
///
/// This component is managed automatically, and should not be accessed
/// manually. It contains data generated from the associated [`ParticleEffect`]
/// component located on the same [`Entity`]. The data is split into this
/// component for change detection reasons, and any change to the associated
/// [`ParticleEffect`] will cause the values of this component to be
/// recalculated. Otherwise the data is cached frame-to-frame for performance.
///
/// All [`ParticleEffect`]s are compiled by the system running in the
/// [`EffectSystems::CompileEffects`] set every frame when they're spawned or
/// when they change, irrelevant of whether the entity if visible
/// ([`Visibility::Visible`]).
#[derive(Debug, Clone, Component)]
pub struct CompiledParticleEffect {
    /// Handle to the underlying asset.
    asset: Handle<EffectAsset>,
    /// Parent effect, if any.
    parent: Option<Entity>,
    /// Child effects.
    children: Vec<Entity>,
    /// Cached simulation condition, to avoid having to query the asset each
    /// time we need it.
    simulation_condition: SimulationCondition,
    /// A custom mesh for this effect, if specified (else an analytic sphere).
    mesh: Option<Handle<bevy_aurora::mesh::AuroraMesh>>,
    /// The Slang source of its kernels, once generated.
    sources: Option<EffectShaderSources>,
    /// Textures used by the effect, if any.
    textures: Vec<Handle<Image>>,
    /// Layout flags.
    layout_flags: LayoutFlags,
    /// Alpha mode.
    alpha_mode: AlphaMode,
    /// Particle layout of the parent effect, if any.
    parent_particle_layout: Option<ParticleLayout>,
    /// PRNG seed.
    prng_seed: u32,
    /// Ready state reported by the render world.
    is_ready: bool,
}

impl Default for CompiledParticleEffect {
    fn default() -> Self {
        Self {
            asset: default(),
            parent: None,
            children: vec![],
            simulation_condition: SimulationCondition::default(),
            mesh: None,
            sources: None,
            textures: vec![],
            layout_flags: LayoutFlags::NONE,
            alpha_mode: default(),
            parent_particle_layout: None,
            prng_seed: 0,
            is_ready: false,
        }
    }
}

impl CompiledParticleEffect {
    /// Check if the effect is ready.
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    #[cfg(test)]
    pub(crate) fn with_ready_for_tests(mut self) -> Self {
        self.is_ready = true;
        self
    }

    /// Clear the compiled data from this component.
    pub(crate) fn clear(&mut self) {
        self.asset = Handle::default();
        self.sources = None;
        self.textures.clear();
    }

    /// Update the compiled effect from its asset and instance.
    pub(crate) fn update(
        &mut self,
        rebuild: bool,
        instance: &ParticleEffect,
        material: Option<&EffectMaterial>,
        mesh: Option<&EffectMesh>,
        asset: &EffectAsset,
        parent_entity: Option<Entity>,
        child_entities: Vec<Entity>,
        parent_layout: Option<ParticleLayout>,
        asset_server: &AssetServer,
    ) {
        trace!(
            "Updating (rebuild:{}) compiled particle effect '{}' ({:?})",
            rebuild,
            asset.name,
            instance.handle,
        );

        // #289 - Panic in fn extract_effects
        // We now keep a strong handle. Since CompiledParticleEffect is kept in sync
        // with the source ParticleEffect, this shouldn't produce any strong cyclic
        // dependency.
        debug_assert!(instance.handle.is_strong());

        // Note: if something marked the ParticleEffect as changed (via Mut for example)
        // but didn't actually change anything, or at least didn't change the asset,
        // then we may end up here with the same asset handle. Don't try to be
        // too smart, and rebuild everything anyway, it's easier than trying to
        // diff what may or may not have changed.
        self.asset = instance.handle.clone();
        self.simulation_condition = asset.simulation_condition;
        self.prng_seed = instance.prng_seed.unwrap_or(asset.prng_seed);

        // Check if the instance changed. If so, rebuild some data from this compiled
        // effect based on the new data of the effect instance.
        if rebuild {
            // Clear the compiled effect if the effect instance changed. We could try to get
            // smarter here, only invalidate what changed, but for now just wipe everything
            // and rebuild from scratch all three shaders together.
            self.sources = None;
        }

        // If the shaders are already compiled, there's nothing more to do
        if self.sources.is_some() {
            return;
        }

        self.parent = parent_entity;
        self.children = child_entities;

        let num_event_bindings = self.children.len() as u32;
        let shader_source = match EffectShaderSources::generate(
            asset,
            parent_layout.as_ref(),
            num_event_bindings,
        ) {
            Ok(shader_source) => shader_source,
            Err(err) => {
                error!(
                    "Failed to generate shaders for effect asset '{}': {}",
                    asset.name, err
                );
                return;
            }
        };

        self.layout_flags = shader_source.layout_flags;
        self.alpha_mode = asset.alpha_mode;
        self.parent_particle_layout = parent_layout;
        trace!(
            "Compiled effect sources: layout_flags={:?} alpha_mode={:?}",
            self.layout_flags,
            self.alpha_mode
        );

        trace!(
            "CompiledParticleEffect::update(): texture_count={} layout_flags={:?}",
            material.map(|mat| mat.images.len()).unwrap_or(0),
            self.layout_flags,
        );
        self.sources = Some(shader_source);

        if let Some(mesh) = mesh {
            self.mesh = Some(mesh.0.clone());
        } else {
            self.mesh = asset
                .mesh
                .as_ref()
                .map(|path: &bevy::asset::AssetPath<'_>| asset_server.load(path));
        }

        self.textures = material.map(|mat| &mat.images).cloned().unwrap_or_default();
    }

    /// The Slang source of the effect's kernels, once generated (debugging, editor).
    pub fn sources(&self) -> Option<&EffectShaderSources> {
        self.sources.as_ref()
    }

    /// The effect asset it was compiled from.
    pub(crate) fn asset(&self) -> &Handle<EffectAsset> {
        &self.asset
    }

    /// The mesh override, if any.
    pub(crate) fn mesh(&self) -> Option<&Handle<bevy_aurora::mesh::AuroraMesh>> {
        self.mesh.as_ref()
    }

    /// The PRNG seed of this frame.
    pub(crate) fn prng_seed(&self) -> u32 {
        self.prng_seed
    }

    /// When the effect simulates.
    pub(crate) fn simulation_condition(&self) -> SimulationCondition {
        self.simulation_condition
    }

    /// Mark the effect ready (its kernels built and storage allocated).
    pub(crate) fn set_ready(&mut self, ready: bool) {
        self.is_ready = ready;
    }
}

const HANABI_COMMON: &str = include_str!("shaders/hanabi_common.slang");
const EFFECT_COMMON_TEMPLATE: &str = include_str!("shaders/effect_common.slang");
const PARTICLES_INIT_SHADER_TEMPLATE: &str = include_str!("shaders/effect_init.slang");
const PARTICLES_UPDATE_SHADER_TEMPLATE: &str = include_str!("shaders/effect_update.slang");

/// What both of an effect's modules start with: its flags, the shared helpers, its particle and
/// property structs and its storage (`effect_common.slang`).
fn effect_common_source(
    local_space: bool,
    attributes_code: &str,
    parent_particle_code: &str,
    properties_code: &str,
) -> String {
    format!(
        "#define LOCAL_SPACE_SIMULATION {}\n\n{HANABI_COMMON}\n{}\n",
        local_space as u32,
        EFFECT_COMMON_TEMPLATE
            .replace("{{ATTRIBUTES}}", attributes_code)
            .replace("{{PARENT_PARTICLE}}", parent_particle_code)
            .replace("{{PROPERTIES}}", properties_code)
    )
}

/// Trait to convert any data structure to its equivalent shader code.
trait ShaderCode {
    /// Generate the shader code for the current state of the object.
    fn to_shader_code(&self, input: &str) -> String;
}

impl ShaderCode for Gradient<Vec2> {
    fn to_shader_code(&self, input: &str) -> String {
        if self.keys().is_empty() {
            return format!("return {};\n", Vec2::ZERO.to_slang_string());
        }
        let mut s: String = self
            .keys()
            .iter()
            .enumerate()
            .map(|(index, key)| {
                format!(
                    "let t{0} = {1};\nlet v{0} = {2};",
                    index,
                    key.ratio().to_slang_string(),
                    key.value.to_slang_string()
                )
            })
            .fold("// Gradient\n".into(), |s, key| s + &key + "\n");
        if self.keys().len() == 1 {
            s + "return v0;\n"
        } else {
            s += &format!("if ({input} <= t0) {{ return v0; }}\n");
            let mut s = self
                .keys()
                .iter()
                .skip(1)
                .enumerate()
                .map(|(index, _key)| {
                    format!(
                        "else if ({input} <= t{1}) {{ return lerp(v{0}, v{1}, ({input} - t{0}) / (t{1} - t{0})); }}\n",
                        index,
                        index + 1
                    )
                })
                .fold(s, |s, key| s + &key);
            let _ = writeln!(s, "else {{ return v{}; }}", self.keys().len() - 1);
            s
        }
    }
}

impl ShaderCode for Gradient<Vec3> {
    fn to_shader_code(&self, input: &str) -> String {
        if self.keys().is_empty() {
            return format!("return {};\n", Vec3::ZERO.to_slang_string());
        }
        let mut s: String = self
            .keys()
            .iter()
            .enumerate()
            .map(|(index, key)| {
                format!(
                    "let t{0} = {1};\nlet v{0} = {2};",
                    index,
                    key.ratio().to_slang_string(),
                    key.value.to_slang_string()
                )
            })
            .fold("// Gradient\n".into(), |s, key| s + &key + "\n");
        if self.keys().len() == 1 {
            s + "return v0;\n"
        } else {
            s += &format!("if ({input} <= t0) {{ return v0; }}\n");
            let mut s = self
                .keys()
                .iter()
                .skip(1)
                .enumerate()
                .map(|(index, _key)| {
                    format!(
                        "else if ({input} <= t{1}) {{ return lerp(v{0}, v{1}, ({input} - t{0}) / (t{1} - t{0})); }}\n",
                        index,
                        index + 1
                    )
                })
                .fold(s, |s, key| s + &key);
            let _ = writeln!(s, "else {{ return v{}; }}", self.keys().len() - 1);
            s
        }
    }
}

impl ShaderCode for Gradient<Vec4> {
    fn to_shader_code(&self, input: &str) -> String {
        if self.keys().is_empty() {
            return format!("return {};\n", Vec4::ZERO.to_slang_string());
        }
        let mut s: String = self
            .keys()
            .iter()
            .enumerate()
            .map(|(index, key)| {
                format!(
                    "let t{0} = {1};\nlet c{0} = {2};",
                    index,
                    key.ratio().to_slang_string(),
                    key.value.to_slang_string()
                )
            })
            .fold("// Gradient\n".into(), |s, key| s + &key + "\n");
        if self.keys().len() == 1 {
            s + "return c0;\n"
        } else {
            s += &format!("if ({input} <= t0) {{ return c0; }}\n");
            let mut s = self
                .keys()
                .iter()
                .skip(1)
                .enumerate()
                .map(|(index, _key)| {
                    format!(
                        "else if ({input} <= t{1}) {{ return lerp(c{0}, c{1}, ({input} - t{0}) / (t{1} - t{0})); }}\n",
                        index,
                        index + 1
                    )
                })
                .fold(s, |s, key| s + &key);
            let _ = writeln!(s, "else {{ return c{}; }}", self.keys().len() - 1);
            s
        }
    }
}

/// Compile all the [`ParticleEffect`] components into a
/// [`CompiledParticleEffect`].
///
/// This system runs in the [`EffectSystems::CompileEffects`] set of the
/// [`PostUpdate`] schedule. It gathers all new instances of [`ParticleEffect`],
/// as well as instances which changed, and (re)compile them into an optimized
/// form saved into the [`CompiledParticleEffect`] component.
///
/// Hidden instances are compiled like visible ones, both to allow users to
/// compile "in the background" by spawning a hidden effect, and also to prevent
/// having mixed state where only some effects are compiled, and effects
/// becoming visible later need to be special casing. If you want to avoid
/// compiling an effect, don't spawn it.
fn compile_effects(
    asset_server: Res<AssetServer>,
    effects: Res<Assets<EffectAsset>>,
    mut q_effects: Query<(
        Entity,
        Ref<ParticleEffect>,
        Option<Ref<EffectMaterial>>,
        Option<Ref<EffectMesh>>,
        Option<Ref<EffectParent>>,
        &mut CompiledParticleEffect,
    )>,
) {
    trace!("compile_effects: {} effect(s)", q_effects.iter().len());

    // Loop over all existing effects and collect the valid ones. We do a separate
    // pass because we can't borrow mutably while doing a double lookup on the
    // query. This map is used to lookup valid parents, and filter out effects with
    // a declared parent but unresolved parent asset.
    let particle_layouts_and_parents: HashMap<Entity, (ParticleLayout, Option<Entity>)> = q_effects
        .iter()
        .filter_map(|(entity, effect, _, _, parent, _)| {
            effects
                .get(&effect.handle)
                .map(|asset| (entity, (asset.particle_layout(), parent.map(|p| p.entity))))
        })
        .collect();

    // Count children
    let mut children: HashMap<Entity, Vec<Entity>> =
        HashMap::with_capacity_and_hasher(particle_layouts_and_parents.len(), Default::default());
    for (child, (_, parent)) in particle_layouts_and_parents.iter() {
        if let Some(parent) = parent.as_ref() {
            children.entry(*parent).or_default().push(*child);
        }
    }

    // Loop over all existing effects to update them, including invisible ones
    for (
        asset,
        entity,
        effect,
        material,
        mesh,
        parent_entity,
        parent_layout,
        mut compiled_effect,
    ) in q_effects.iter_mut().filter_map(
        |(entity, effect, material, mesh, parent, compiled_effect)| {
            // Check if asset is available, otherwise silently ignore as we can't check for
            // changes, and conceptually it makes no sense to render a particle effect whose
            // asset was unloaded.
            let asset = effects.get(&effect.handle)?;

            // Same for the parent asset, if any.
            let (parent_entity, parent_layout) = if let Some(parent) = &parent {
                let Some((parent_layout, _)) = particle_layouts_and_parents.get(&parent.entity)
                else {
                    // There's a parent declared, but not found. Skip the current asset.
                    return None;
                };
                // Declared parent with found parent asset, child asset is valid.
                (Some(parent.entity), Some(parent_layout.clone()))
            } else {
                // No declared parent, asset is valid.
                (None, None)
            };

            Some((
                asset,
                entity,
                effect,
                material,
                mesh,
                parent_entity,
                parent_layout,
                compiled_effect,
            ))
        },
    ) {
        let child_entities = children
            .get_mut(&entity)
            .map(std::mem::take)
            .unwrap_or_default();

        // If the ParticleEffect didn't change, and the compiled one is for the correct
        // asset, then there's nothing to do.
        let material_changed = material.as_ref().is_some_and(|r| r.is_changed());
        let need_rebuild =
            effect.is_changed() || material_changed || compiled_effect.children != child_entities;
        if need_rebuild || (compiled_effect.asset != effect.handle) {
            if need_rebuild {
                debug!("Invalidating the compiled cache for effect on entity {:?} due to changes in the ParticleEffect component. If you see this message too much, then performance might be affected. Find why the change detection of the ParticleEffect is triggered.", entity);
            }

            compiled_effect.update(
                need_rebuild,
                &effect,
                material.map(|r| r.into_inner()),
                mesh.map(|r| r.into_inner()),
                asset,
                parent_entity,
                child_entities,
                parent_layout,
                &asset_server,
            );
        } else {
            // Update the PRNG seed. Unfortunately at the minute the "seed" (which
            // really is the internal PRNG state rather) is not cached on GPU, and
            // is re-uploaded each frame, so if it's not changed every frame then
            // there's no randomness anymore, because the uses of the previous frame
            // are "forgotten".
            let mut rng = rand::rngs::StdRng::seed_from_u64(compiled_effect.prng_seed as u64);
            compiled_effect.prng_seed = rng.random();
        }
    }

    // Clear removed effects, to allow them to be released by the asset server
    for (_, effect, _, _, parent, mut compiled_effect) in q_effects.iter_mut() {
        // If the effect has no asset, clear its compilation
        if effects.get(&effect.handle).is_none() {
            compiled_effect.clear();
        }

        // If the effect has a parent, and that parent has no asset, also clear the
        // child's compilation.
        if let Some(parent) = parent {
            if particle_layouts_and_parents.get(&parent.entity).is_none() {
                compiled_effect.clear();
            }
        }
    }
}

/// Update all properties of a [`ParticleEffect`] into its associated
/// [`EffectProperties`].
///
/// This system runs in the [`EffectSystems::UpdatePropertiesFromAsset`] set of
/// the [`PostUpdate`] schedule. It gathers all new instances of
/// [`ParticleEffect`], as well as instances which changed, and update their
/// associated [`EffectProperties`] component.
///
/// Hidden instances are processed like visible ones, both to allow users to
/// compile "in the background" by spawning a hidden effect, and also to prevent
/// having mixed state where only some effects are compiled, and effects
/// becoming visible later need to be special casing. If you want to avoid
/// compiling an effect, don't spawn it.
fn update_properties_from_asset(
    assets: Res<Assets<EffectAsset>>,
    mut q_effects: Query<(Ref<ParticleEffect>, &mut EffectProperties), Changed<ParticleEffect>>,
) {
    #[cfg(feature = "trace")]
    let _span = bevy::log::info_span!("update_properties_from_asset").entered();
    trace!("update_properties_from_asset()");

    // Loop over all existing effects, including invisible ones
    for (effect, properties) in q_effects.iter_mut() {
        // Check if the asset is available, otherwise silently ignore as we can't check
        // for changes, and conceptually it makes no sense to render a particle
        // effect whose asset was unloaded.
        let Some(asset) = assets.get(&effect.handle) else {
            continue;
        };

        EffectProperties::update(properties, asset.properties(), effect.is_added());
    }
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use bevy::{
        asset::{
            io::{
                memory::{Dir, MemoryAssetReader},
                AssetSourceBuilder, AssetSourceBuilders, AssetSourceId,
            },
            AssetServerMode, UnapprovedPathMode,
        },
        camera::visibility::{VisibilityPlugin, VisibilitySystems},
        tasks::{IoTaskPool, TaskPoolBuilder},
    };

    use super::*;
    use crate::spawn::new_rng;

    const INTS: &[usize] = &[1, 2, 4, 8, 9, 15, 16, 17, 23, 24, 31, 32, 33];
    const INTS_POW2: &[usize] = &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

    /// Same as `INTS`, rounded up to 16
    const INTS16: &[usize] = &[16, 16, 16, 16, 16, 16, 16, 32, 32, 32, 32, 32, 48];

    #[test]
    fn next_multiple() {
        // align-1 is no-op
        for &size in INTS {
            assert_eq!(size, size.next_multiple_of(1));
        }

        // zero-sized is always aligned
        for &align in INTS_POW2 {
            assert_eq!(0, 0usize.next_multiple_of(align));
        }

        // size < align : rounds up to align
        for &size in INTS {
            assert_eq!(256, size.next_multiple_of(256));
        }

        // size > align : actually aligns
        for (&size, &aligned_size) in INTS.iter().zip(INTS16) {
            assert_eq!(aligned_size, size.next_multiple_of(16));
        }
    }

    #[test]
    fn to_wgsl_f32() {
        let s = 1.0_f32.to_slang_string();
        assert_eq!(s, "1.");
        let s = (-1.0_f32).to_slang_string();
        assert_eq!(s, "-1.");
        let s = 1.5_f32.to_slang_string();
        assert_eq!(s, "1.5");
        let s = 0.5_f32.to_slang_string();
        assert_eq!(s, "0.5");
        let s = 0.123_456_78_f32.to_slang_string();
        assert_eq!(s, "0.123457"); // 6 digits
    }

    #[test]
    fn to_wgsl_f64() {
        let s = 1.0_f64.to_slang_string();
        assert_eq!(s, "1.");
        let s = (-1.0_f64).to_slang_string();
        assert_eq!(s, "-1.");
        let s = 1.5_f64.to_slang_string();
        assert_eq!(s, "1.5");
        let s = 0.5_f64.to_slang_string();
        assert_eq!(s, "0.5");
        let s = 0.123_456_789_012_345_67_f64.to_slang_string();
        assert_eq!(s, "0.123456789012346"); // 15 digits
    }

    #[test]
    fn to_wgsl_vec() {
        let s = Vec2::new(1., 2.).to_slang_string();
        assert_eq!(s, "float2(1.,2.)");
        let s = Vec3::new(1., 2., -1.).to_slang_string();
        assert_eq!(s, "float3(1.,2.,-1.)");
        let s = Vec4::new(1., 2., -1., 2.).to_slang_string();
        assert_eq!(s, "float4(1.,2.,-1.,2.)");
    }

    #[test]
    fn to_wgsl_ivec() {
        let s = IVec2::new(1, 2).to_slang_string();
        assert_eq!(s, "int2(1,2)");
        let s = IVec3::new(1, 2, -1).to_slang_string();
        assert_eq!(s, "int3(1,2,-1)");
        let s = IVec4::new(1, 2, -1, 2).to_slang_string();
        assert_eq!(s, "int4(1,2,-1,2)");
    }

    #[test]
    fn to_wgsl_uvec() {
        let s = UVec2::new(1, 2).to_slang_string();
        assert_eq!(s, "uint2(1u,2u)");
        let s = UVec3::new(1, 2, 42).to_slang_string();
        assert_eq!(s, "uint3(1u,2u,42u)");
        let s = UVec4::new(1, 2, 42, 5).to_slang_string();
        assert_eq!(s, "uint4(1u,2u,42u,5u)");
    }

    #[test]
    fn to_wgsl_bvec() {
        let s = BVec2::new(false, true).to_slang_string();
        assert_eq!(s, "bool2(false,true)");
        let s = BVec3::new(false, true, true).to_slang_string();
        assert_eq!(s, "bool3(false,true,true)");
        let s = BVec4::new(false, true, true, false).to_slang_string();
        assert_eq!(s, "bool4(false,true,true,false)");
    }

    #[test]
    fn to_wgsl_value_f32() {
        let s = CpuValue::Single(1.0_f32).to_slang_string();
        assert_eq!(s, "1.");
        let s = CpuValue::Uniform((1.0_f32, 2.0_f32)).to_slang_string();
        assert_eq!(s, "(frand() * (2. - 1.) + 1.)");
    }

    #[test]
    fn to_wgsl_value_vec2() {
        let s = CpuValue::Single(Vec2::ONE).to_slang_string();
        assert_eq!(s, "float2(1.,1.)");
        let s = CpuValue::Uniform((Vec2::ZERO, Vec2::ONE)).to_slang_string();
        assert_eq!(
            s,
            "(frand2() * (float2(1.,1.) - float2(0.,0.)) + float2(0.,0.))"
        );
    }

    #[test]
    fn to_wgsl_value_vec3() {
        let s = CpuValue::Single(Vec3::ONE).to_slang_string();
        assert_eq!(s, "float3(1.,1.,1.)");
        let s = CpuValue::Uniform((Vec3::ZERO, Vec3::ONE)).to_slang_string();
        assert_eq!(
            s,
            "(frand3() * (float3(1.,1.,1.) - float3(0.,0.,0.)) + float3(0.,0.,0.))"
        );
    }

    #[test]
    fn to_wgsl_value_vec4() {
        let s = CpuValue::Single(Vec4::ONE).to_slang_string();
        assert_eq!(s, "float4(1.,1.,1.,1.)");
        let s = CpuValue::Uniform((Vec4::ZERO, Vec4::ONE)).to_slang_string();
        assert_eq!(s, "(frand4() * (float4(1.,1.,1.,1.) - float4(0.,0.,0.,0.)) + float4(0.,0.,0.,0.))");
    }

    #[test]
    fn to_shader_code() {
        let mut grad = Gradient::new();
        assert_eq!("return float4(0.,0.,0.,0.);\n", grad.to_shader_code("key"));

        grad.add_key(0.0, Vec4::splat(0.0));
        assert_eq!(
            "// Gradient\nlet t0 = 0.;\nlet c0 = float4(0.,0.,0.,0.);\nreturn c0;\n",
            grad.to_shader_code("key")
        );

        grad.add_key(1.0, Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(
            r#"// Gradient
let t0 = 0.;
let c0 = float4(0.,0.,0.,0.);
let t1 = 1.;
let c1 = float4(1.,0.,0.,1.);
if (key <= t0) { return c0; }
else if (key <= t1) { return lerp(c0, c1, (key - t0) / (t1 - t0)); }
else { return c1; }
"#,
            grad.to_shader_code("key")
        );
    }

    #[test]
    fn test_simulation_space_eval() {
        let particle_layout = ParticleLayout::empty();
        let property_layout = PropertyLayout::default();
        let texture_layout = TextureLayout::default();
        {
            // Global init requires storing the particle's position
            let ctx = ShaderWriter::new(
                ModifierContext::Init,
                &property_layout,
                &particle_layout,
                &texture_layout,
            );
            assert!(SimulationSpace::Local.eval(&ctx).is_ok());
            assert!(SimulationSpace::Global.eval(&ctx).is_err());

            let particle_layout = ParticleLayout::new().append(Attribute::POSITION).build();
            let ctx = ShaderWriter::new(
                ModifierContext::Init,
                &property_layout,
                &particle_layout,
                &texture_layout,
            );
            assert!(SimulationSpace::Local.eval(&ctx).is_ok());
            assert!(SimulationSpace::Global.eval(&ctx).is_ok());
        }
        {
            let ctx = ShaderWriter::new(
                ModifierContext::Update,
                &property_layout,
                &particle_layout,
                &texture_layout,
            );
            assert_eq!(SimulationSpace::Local.eval(&ctx).unwrap(), "");
            assert_eq!(SimulationSpace::Global.eval(&ctx).unwrap(), "");

            let particle_layout = ParticleLayout::new().append(Attribute::POSITION).build();
            let ctx = ShaderWriter::new(
                ModifierContext::Update,
                &property_layout,
                &particle_layout,
                &texture_layout,
            );
            assert_eq!(SimulationSpace::Local.eval(&ctx).unwrap(), "");
            assert_eq!(SimulationSpace::Global.eval(&ctx).unwrap(), "");
        }
        {
            // In the render context, the particle position is always available (either
            // stored or not), so the simulation space can always be evaluated.
            let texture_layout = TextureLayout::default();
            let ctx = RenderContext::new(&property_layout, &particle_layout, &texture_layout);
            assert!(SimulationSpace::Local.eval(&ctx).is_ok());
            assert!(SimulationSpace::Global.eval(&ctx).is_ok());
        }
    }

    #[test]
    fn test_simulation_space_init_spawn_to_sim() {
        let property_layout = PropertyLayout::default();
        let texture_layout = TextureLayout::default();
        let particle_layout = ParticleLayout::new()
            .append(Attribute::POSITION)
            .append(Attribute::VELOCITY)
            .build();
        let ctx = ShaderWriter::new(
            ModifierContext::Init,
            &property_layout,
            &particle_layout,
            &texture_layout,
        );

        let global = SimulationSpace::Global.eval(&ctx).unwrap();
        assert!(global.contains("float4(particle.position, 1.0)"));
        assert!(global.contains("float4(particle.velocity, 0.0)"));
        assert!(!global.contains("transform[3]"));

        let local = SimulationSpace::Local.eval(&ctx).unwrap();
        assert!(local.is_empty());
    }

    fn make_test_app() -> App {
        IoTaskPool::get_or_init(|| {
            TaskPoolBuilder::default()
                .num_threads(1)
                .thread_name("Hanabi test IO Task Pool".to_string())
                .build()
        });

        let mut app = App::new();

        let watch_for_changes = false;
        let mut builders = app
            .world_mut()
            .get_resource_or_insert_with::<AssetSourceBuilders>(Default::default);
        let dir = Dir::default();
        let dummy_builder =
            AssetSourceBuilder::new(move || Box::new(MemoryAssetReader { root: dir.clone() }));
        builders.insert(AssetSourceId::Default, dummy_builder);
        let sources = builders.build_sources(watch_for_changes, false);
        let asset_server = AssetServer::new(
            sources.into(),
            AssetServerMode::Unprocessed,
            watch_for_changes,
            UnapprovedPathMode::Forbid,
        );

        app.insert_resource(asset_server);
        // app.add_plugins(DefaultPlugins);
        app.init_asset::<Mesh>();
        app.init_asset::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>();
        app.add_plugins(VisibilityPlugin);
        app.insert_resource(Random(new_rng()));
        app.init_asset::<EffectAsset>();
        app.add_systems(
            PostUpdate,
            compile_effects.after(VisibilitySystems::CheckVisibility),
        );

        app
    }

    /// Test case for `tick_spawners()`.
    struct TestCase {
        /// Initial entity visibility on spawn. If `None`, do not add a
        /// [`Visibility`] component.
        visibility: Option<Visibility>,
    }

    impl TestCase {
        fn new(visibility: Option<Visibility>) -> Self {
            Self { visibility }
        }
    }

    // Regression test for #343
    #[test]
    fn test_compile_effect_invalid_handle() {
        let mut app = make_test_app();

        let effect_entity = {
            let world = app.world_mut();

            // Spawn particle effect
            let entity = world.spawn(ParticleEffect::default()).id();

            // Spawn a camera, otherwise ComputedVisibility stays at HIDDEN
            world.spawn(Camera3d::default());

            entity
        };

        // Tick once
        app.update();

        // Check
        {
            let world = app.world_mut();

            let (entity, particle_effect, compiled_particle_effect) = world
                .query::<(Entity, &ParticleEffect, &CompiledParticleEffect)>()
                .iter(world)
                .next()
                .unwrap();
            assert_eq!(entity, effect_entity);
            assert_eq!(particle_effect.handle, Handle::<EffectAsset>::default());

            // `compile_effects()` cannot update the CompiledParticleEffect because the
            // asset is invalid
            assert_eq!(
                compiled_particle_effect.asset,
                Handle::<EffectAsset>::default()
            );
            assert!(compiled_particle_effect.sources.is_none());
        }
    }

    // Regression test for #228
    #[test]
    fn test_compile_effect_changed() {
        let spawner = SpawnerSettings::once(32.0.into());

        let mut app = make_test_app();

        let (effect_entity, handle) = {
            let world = app.world_mut();

            // Add effect asset
            let mut assets = world.resource_mut::<Assets<EffectAsset>>();
            let mut module = Module::default();
            let init_pos = module.lit(Vec3::ZERO);
            let mut asset = EffectAsset::new(64, spawner, module)
                .init(SetAttributeModifier::new(Attribute::POSITION, init_pos));
            asset.simulation_condition = SimulationCondition::Always;
            let handle = assets.add(asset);

            // Spawn particle effect
            let entity = world
                .spawn((
                    ParticleEffect::new(handle.clone()),
                    CompiledParticleEffect::default(),
                ))
                .id();

            // Spawn a camera, otherwise ComputedVisibility stays at HIDDEN
            world.spawn(Camera3d::default());

            (entity, handle)
        };

        // Tick once
        app.update();

        // Check
        {
            let world = app.world_mut();

            let (entity, particle_effect, compiled_particle_effect) = world
                .query::<(Entity, &ParticleEffect, &CompiledParticleEffect)>()
                .iter(world)
                .next()
                .unwrap();
            assert_eq!(entity, effect_entity);
            assert_eq!(particle_effect.handle, handle);

            // `compile_effects()` always updates the CompiledParticleEffect
            assert_eq!(compiled_particle_effect.asset, handle);
            assert!(compiled_particle_effect.asset.is_strong());
            assert!(compiled_particle_effect.sources.is_some());
        }

        // Mark as changed without actually changing anything
        {
            let world = app.world_mut();

            let mut particle_effect = world
                .query::<&mut ParticleEffect>()
                .iter_mut(world)
                .next()
                .unwrap();

            // Force via Mut to mark the component as changed
            particle_effect.deref_mut();
        }

        // Tick once - Regression test for #228, this should not panic
        app.update();

        // Check again, nothing changed
        {
            let world = app.world_mut();

            let (entity, particle_effect, compiled_particle_effect) = world
                .query::<(Entity, &ParticleEffect, &CompiledParticleEffect)>()
                .iter(world)
                .next()
                .unwrap();
            assert_eq!(entity, effect_entity);
            assert_eq!(particle_effect.handle, handle);

            // `compile_effects()` always updates the CompiledParticleEffect
            assert_eq!(compiled_particle_effect.asset, handle);
            assert!(compiled_particle_effect.asset.is_strong());
            assert!(compiled_particle_effect.sources.is_some());
        }
    }

    #[test]
    fn test_compile_effect_visibility() {
        let spawner = SpawnerSettings::once(32.0.into());

        for test_case in &[
            TestCase::new(None),
            TestCase::new(Some(Visibility::Hidden)),
            TestCase::new(Some(Visibility::Visible)),
        ] {
            let mut app = make_test_app();

            let (effect_entity, handle) = {
                let world = app.world_mut();

                // Add effect asset
                let mut assets = world.resource_mut::<Assets<EffectAsset>>();
                let mut module = Module::default();
                let init_pos = module.lit(Vec3::ZERO);
                let mut asset = EffectAsset::new(64, spawner, module)
                    .init(SetAttributeModifier::new(Attribute::POSITION, init_pos));
                asset.simulation_condition = if test_case.visibility.is_some() {
                    SimulationCondition::WhenVisible
                } else {
                    SimulationCondition::Always
                };
                // Use local simulation space so we don't need to store Attribute::POSITION for
                // particles
                asset.simulation_space = SimulationSpace::Local;
                let handle = assets.add(asset);

                // Spawn particle effect
                let entity = if let Some(visibility) = test_case.visibility {
                    world
                        .spawn((
                            visibility,
                            InheritedVisibility::default(),
                            ParticleEffect::new(handle.clone()),
                            CompiledParticleEffect::default(),
                        ))
                        .id()
                } else {
                    world
                        .spawn((
                            ParticleEffect::new(handle.clone()),
                            CompiledParticleEffect::default(),
                        ))
                        .id()
                };

                // Spawn a camera, otherwise ComputedVisibility stays at HIDDEN
                world.spawn(Camera3d::default());

                (entity, handle)
            };

            // Tick once
            app.update();

            let world = app.world_mut();

            // Check the state of the components after `tick_spawners()` ran
            if let Some(test_visibility) = test_case.visibility {
                // Simulated-when-visible effect (SimulationCondition::WhenVisible)

                let (
                    entity,
                    visibility,
                    inherited_visibility,
                    particle_effect,
                    compiled_particle_effect,
                ) = world
                    .query::<(
                        Entity,
                        &Visibility,
                        &InheritedVisibility,
                        &ParticleEffect,
                        &CompiledParticleEffect,
                    )>()
                    .iter(world)
                    .next()
                    .unwrap();
                assert_eq!(entity, effect_entity);
                assert_eq!(visibility, test_visibility);
                assert_eq!(
                    inherited_visibility.get(),
                    test_visibility == Visibility::Visible
                );
                assert_eq!(particle_effect.handle, handle);

                // `compile_effects()` always updates the CompiledParticleEffect of new effects,
                // even if hidden
                assert_eq!(compiled_particle_effect.asset, handle);
                assert!(compiled_particle_effect.asset.is_strong());
                assert!(compiled_particle_effect.sources.is_some());

                // Toggle visibility and tick once more; this shouldn't panic (regression; #182)
                let (mut visibility, _) = world
                    .query::<(&mut Visibility, &ParticleEffect)>()
                    .iter_mut(world)
                    .next()
                    .unwrap();
                if *visibility == Visibility::Visible {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Visible;
                }
                app.update();
            } else {
                // Always-simulated effect (SimulationCondition::Always)

                let (entity, particle_effect, compiled_particle_effect) = world
                    .query::<(Entity, &ParticleEffect, &CompiledParticleEffect)>()
                    .iter(world)
                    .next()
                    .unwrap();
                assert_eq!(entity, effect_entity);
                assert_eq!(particle_effect.handle, handle);

                // `compile_effects()` always updates the CompiledParticleEffect
                assert_eq!(compiled_particle_effect.asset, handle);
                assert!(compiled_particle_effect.asset.is_strong());
                assert!(compiled_particle_effect.sources.is_some());
            }
        }
    }

    #[test]
    fn slotdim_is() {
        for dim in [
            SlotDimension::D1,
            SlotDimension::D2,
            SlotDimension::D2Array,
            SlotDimension::Cube,
            SlotDimension::CubeArray,
            SlotDimension::D3,
            SlotDimension::DepthD2,
            SlotDimension::DepthD2Array,
            SlotDimension::DepthCube,
            SlotDimension::DepthCubeArray,
        ] {
            // The canonical WGSL name, which is what the SlotDimentions debug-format to,
            // happens to always contain "array" if the texture is an array texture, "depth"
            // if it's used for comparison, and "cube" if it's a cube texture. We use this
            // as validation.
            let name = format!("{:?}", dim).to_ascii_lowercase();

            let is_array = name.contains("array");
            assert_eq!(is_array, dim.is_array());

            let is_depth = name.contains("depth");
            assert_eq!(is_depth, dim.is_depth());

            let is_cube = name.contains("cube");
            assert_eq!(is_cube, dim.is_cube());
        }
    }

}
