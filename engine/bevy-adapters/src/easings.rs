//! bevy_easings (docs/plugins.feature PL0a). Its easing, EasingComponent<T>, is generic over the value it eases and
//! isn't reflected (its timer and ease method are private, the method a fn pointer at worst), so nothing spawned from
//! TypeScript could ease. What had to be written by hand: two reflected components of ours, each turned into the
//! crate's EasingComponent when it's added or changed, from the entity's value now to the one given:
//! `EaseTransform` (its Transform) and `EaseSpriteColor` (its Sprite's colour). bevy_easings runs them on Update.

use std::time::Duration;

use bevy::prelude::*;
use bevy_easings::{Ease, EaseMethod, EasingType};

/// How an easing repeats.
#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq)]
pub enum EaseRepeat {
    /// once, and it stays at the end
    #[default]
    Once,
    /// from the start again each time
    Loop,
    /// there and back
    PingPong,
}

/// Eases the entity's Transform to `to` over `seconds`, from where it is when this is added or changed.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
pub struct EaseTransform {
    pub to: Transform,
    pub seconds: f32,
    /// Bevy's easing curves: "Linear" and the ones bevy_easings has too ("QuadraticInOut", "BounceOut", ...; see method)
    pub function: EaseFunction,
    pub repeat: EaseRepeat,
}

/// a second, linearly, once, to `$to`
macro_rules! ease_default {
    ($t:ty, $to:expr) => {
        impl Default for $t {
            fn default() -> Self {
                Self { to: $to, seconds: 1.0, function: EaseFunction::Linear, repeat: EaseRepeat::Once }
            }
        }
    };
}

ease_default!(EaseTransform, Transform::IDENTITY);

/// Eases the entity's Sprite colour to `to` over `seconds`, from its colour when this is added or changed.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
pub struct EaseSpriteColor {
    pub to: Color,
    pub seconds: f32,
    pub function: EaseFunction,
    pub repeat: EaseRepeat,
}

ease_default!(EaseSpriteColor, Color::WHITE);

fn easing_type(seconds: f32, repeat: EaseRepeat) -> EasingType {
    let duration = Duration::from_secs_f32(seconds.max(0.0));
    match repeat {
        EaseRepeat::Once => EasingType::Once { duration },
        EaseRepeat::Loop => EasingType::Loop { duration, pause: None },
        EaseRepeat::PingPong => EasingType::PingPong { duration, pause: None },
    }
}

/// Bevy's EaseFunction (reflected) as bevy_easings' own (interpolation's, which isn't): Linear and the 30 curves both
/// name the same way; Bevy's others (Smoothstep, Steps, ...) have no match there and ease linearly, with a warning.
fn method(f: EaseFunction) -> EaseMethod {
    use bevy_easings::EaseFunction as E;
    macro_rules! same {
        ($($n:ident)*) => {
            match f {
                EaseFunction::Linear => EaseMethod::Linear,
                $(EaseFunction::$n => EaseMethod::EaseFunction(E::$n),)*
                other => {
                    warn!("easings: bevy_easings has no {other:?}; easing linearly");
                    EaseMethod::Linear
                }
            }
        };
    }
    same!(QuadraticIn QuadraticOut QuadraticInOut CubicIn CubicOut CubicInOut QuarticIn QuarticOut QuarticInOut
        QuinticIn QuinticOut QuinticInOut SineIn SineOut SineInOut CircularIn CircularOut CircularInOut ExponentialIn
        ExponentialOut ExponentialInOut ElasticIn ElasticOut ElasticInOut BackIn BackOut BackInOut BounceIn BounceOut
        BounceInOut)
}

pub(crate) fn install(app: &mut App) {
    if !app.is_plugin_added::<bevy_easings::EasingsPlugin<()>>() {
        app.add_plugins(bevy_easings::EasingsPlugin::<()>::default());
    }
    app.register_type::<EaseTransform>().register_type::<EaseSpriteColor>();
    app.add_systems(Update, (ease_transforms, ease_sprite_colors).before(bevy_easings::EasingsLabel));
}

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "easings", install, group: None, on_demand: false });
}

fn ease_transforms(changed: Query<(Entity, &EaseTransform, &Transform), Changed<EaseTransform>>, mut commands: Commands) {
    for (e, ease, now) in &changed {
        commands.entity(e).insert(now.ease_to(ease.to, method(ease.function), easing_type(ease.seconds, ease.repeat)));
    }
}

fn ease_sprite_colors(changed: Query<(Entity, &EaseSpriteColor, &Sprite), Changed<EaseSpriteColor>>, mut commands: Commands) {
    for (e, ease, now) in &changed {
        let to = Sprite { color: ease.to, ..now.clone() };
        commands.entity(e).insert(now.clone().ease_to(to, method(ease.function), easing_type(ease.seconds, ease.repeat)));
    }
}
