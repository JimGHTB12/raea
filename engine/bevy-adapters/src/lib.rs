//! Adapters for declared Bevy plugins whose API isn't reflected (docs/plugins.feature PL7, PL8): a plugin that keeps
//! its state in assets or builders TypeScript can't name gets a reflected component of ours (enoki's Sparks,
//! spritesheet's SheetAnimation) that builds the plugin's own types when it's added. A declaration's
//! `"adapter": "<module>"` names a module here when its file is (else one of raea-bevy-plugins' own, for adapters
//! that need its bridge); tools/bevy-plugins.sh sync writes the module lines and this crate's dependencies by feature,
//! and raea-bevy-plugins' install calls the module's `register()`, which registers its plugin with raea-render's hooks.

/// rapier3d's and rapier2d's install and register, the same but for the crate each module imports: rapier's plugin
/// (added once: every registered plugin shares one App), the module's ColliderShape and its build_colliders system
#[cfg(any(feature = "rapier3d", feature = "rapier2d"))]
macro_rules! rapier_adapter {
    ($name:literal) => {
        pub(crate) fn install(app: &mut App) {
            if !app.is_plugin_added::<RapierPhysicsPlugin<NoUserData>>() {
                app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
            }
            app.register_type::<ColliderShape>();
            app.add_systems(Update, build_colliders);
        }

        pub fn register() {
            raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: $name, install, group: None, on_demand: false });
        }
    };
}

// the adapters a declaration names, by feature
// >>> declared modules: written by tools/bevy-plugins.sh sync; don't edit
#[cfg(feature = "easings")]
pub mod easings;
#[cfg(feature = "enoki")]
pub mod enoki;
#[cfg(feature = "ldtk")]
pub mod ldtk;
#[cfg(feature = "light_2d")]
pub mod light_2d;
#[cfg(feature = "rapier2d")]
pub mod rapier2d;
#[cfg(feature = "rapier3d")]
pub mod rapier3d;
#[cfg(feature = "spritesheet")]
pub mod spritesheet;
#[cfg(feature = "text3d")]
pub mod text3d;
#[cfg(feature = "tilemap")]
pub mod tilemap;
#[cfg(feature = "tween")]
pub mod tween;
// <<< declared modules
