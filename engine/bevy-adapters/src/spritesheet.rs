//! bevy_spritesheet_animation (sprite-sheet animation). What had to be written by hand: its Animation asset is built
//! by a builder over crate-private fields and the sprite's atlas layout by its Spritesheet, so neither can be given
//! inline; `SheetAnimation` is a reflected component of ours that builds both from plain numbers when it's added. It
//! draws as a Sprite in the scene's 2D overlay (Draws2d brings its camera up; pixels from the frame's top left, +y up).

use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;

/// A sprite playing cells of a sheet in a loop, built into a Sprite and a SpritesheetAnimation when added.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
#[require(raea_sys_sprite::Draws2d)]
pub struct SheetAnimation {
    pub image: Handle<Image>,
    /// the image's size in pixels (the atlas is laid out before the image loads)
    pub image_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    /// the cells played, numbered row by row from the top left
    pub frames: Vec<usize>,
    /// milliseconds each cell shows
    pub frame_ms: u32,
    /// its size on the frame in pixels
    pub size: Vec2,
}

impl Default for SheetAnimation {
    fn default() -> Self {
        SheetAnimation { image: Handle::default(), image_size: UVec2::ONE, columns: 1, rows: 1, frames: vec![0], frame_ms: 100, size: Vec2::splat(32.0) }
    }
}

pub(crate) fn install(app: &mut App) {
    app.add_plugins(SpritesheetAnimationPlugin);
    app.register_type::<SheetAnimation>();
    app.add_systems(Update, build_sheets);
}

fn build_sheets(added: Query<(Entity, &SheetAnimation), Added<SheetAnimation>>, mut animations: ResMut<Assets<Animation>>, mut layouts: ResMut<Assets<TextureAtlasLayout>>, mut commands: Commands) {
    for (e, s) in &added {
        let sheet = Spritesheet::new(&s.image, s.columns, s.rows);
        let animation = sheet.create_animation().add_indices(s.frames.iter().copied()).set_duration(AnimationDuration::PerFrame(s.frame_ms)).build();
        let mut sprite = sheet.with_size_hint(s.image_size.x, s.image_size.y).sprite(&mut layouts);
        sprite.custom_size = Some(s.size);
        commands.entity(e).insert((sprite, SpritesheetAnimation::new(animations.add(animation))));
    }
}

/// The adapter's registration (the declaration's `"adapter": "spritesheet"`).
pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "spritesheet", install, group: None, on_demand: false });
}
