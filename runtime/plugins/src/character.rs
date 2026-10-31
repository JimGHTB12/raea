//! A character's goal and where the motion engine has it (docs/sdk.feature S6e): reflected components, so TypeScript's
//! CharacterPlugin (sdk/character/src/plugin.ts) reads the goal and writes the pose through BRP as any component.
//! The motion engine is TypeScript's (perform is frame-pure): the World holds what a character is asked to do and
//! where it is, and the joints are a function of the two.

use super::*;

/// What a character is asked to do: a registered motion library's name, the performance (a PerformanceSpec as
/// JSON) and the App's frame it starts at. A goal brings its CharacterPose.
#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
#[require(CharacterPose)]
pub struct CharacterGoal {
    pub motion: String,
    pub spec: String,
    pub start: u32,
}

/// Where the motion engine has the character this frame: the performance's frame, its place on the floor, its yaw
#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct CharacterPose {
    pub frame: u32,
    pub translation: Vec3,
    pub yaw: f32,
}

/// The two registered for reflection, so TypeScript names them by path
pub fn register(app: &mut App) {
    app.register_type::<CharacterGoal>().register_type::<CharacterPose>();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A goal inserted by its path (as TypeScript's a.insert sends it) brings a default pose, and the pose written
    /// by path reads back
    #[test]
    fn a_goal_by_path_brings_its_pose() {
        let mut app = App::new();
        register(&mut app);
        let w = app.world_mut();
        let e = w.spawn_empty().id();
        insert_reflect(w, e, "raea_runtime_plugins::character::CharacterGoal", &json!({"motion": "kid", "spec": "{}", "start": 4})).unwrap();
        assert_eq!(w.get::<CharacterGoal>(e).map(|g| (g.motion.as_str(), g.start)), Some(("kid", 4)));
        assert_eq!(w.get::<CharacterPose>(e), Some(&CharacterPose::default()));
        insert_reflect(w, e, "raea_runtime_plugins::character::CharacterPose", &json!({"frame": 3, "translation": [1.0, 0.0, 2.0], "yaw": 0.5})).unwrap();
        assert_eq!(w.get::<CharacterPose>(e), Some(&CharacterPose { frame: 3, translation: Vec3::new(1.0, 0.0, 2.0), yaw: 0.5 }));
    }
}
