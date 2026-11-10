//! A free camera over a scene (docs/preview.feature PV1c): while `FreeView` holds an orbit, the frame's camera is put
//! there after the composition's rows have placed it and before transforms propagate, so the frame is drawn (and
//! picked: pick.rs reads the same camera) from the orbit. Cleared, the camera goes back to where the composition last
//! put it. Nothing in the composition's rows changes.

use bevy::prelude::*;

/// An orbit about `target`: `yaw` about +Y from +Z, `pitch` up from the ground plane (radians), `distance` metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
}

impl Orbit {
    pub fn transform(&self) -> Transform {
        let pitch = self.pitch.clamp(-1.553, 1.553);
        let dir = Vec3::new(self.yaw.sin() * pitch.cos(), pitch.sin(), self.yaw.cos() * pitch.cos());
        Transform::from_translation(self.target + dir * self.distance.max(1e-3)).looking_at(self.target, Vec3::Y)
    }
}

/// The orbit the frame is drawn from, if any; and what the override last wrote and what the composition had put
/// there, to tell the composition's own moves from the override's and to put its camera back.
#[derive(Resource, Default)]
pub struct FreeView {
    pub orbit: Option<Orbit>,
    wrote: Option<Transform>,
    own: Option<Transform>,
}

/// PostUpdate, before transforms propagate.
pub fn apply(mut view: ResMut<FreeView>, mut cams: Query<(&Camera, &mut Transform), With<Camera3d>>) {
    // the frame's camera, as pick.rs and layer.rs choose it
    let Some(mut t) = cams.iter_mut().filter(|(c, _)| c.is_active).min_by_key(|(c, _)| c.order).map(|(_, t)| t) else { return };
    // a transform that isn't what the override wrote is the composition's own, placed since
    if view.wrote != Some(*t) {
        view.own = Some(*t);
    }
    match view.orbit {
        Some(o) => {
            let to = o.transform();
            if *t != to {
                *t = to;
            }
            view.wrote = Some(to);
        }
        None => {
            if let (Some(_), Some(own)) = (view.wrote.take(), view.own) {
                *t = own;
            }
        }
    }
}

/// The orbit that looks at the scene as its camera does now: the same place, about the point `ahead` metres along
/// its view (the scene's middle when the caller knows it).
pub fn orbit_of(camera: &GlobalTransform, target: Vec3) -> Orbit {
    let off = camera.translation() - target;
    let distance = off.length().max(1e-3);
    Orbit { yaw: off.x.atan2(off.z), pitch: (off.y / distance).clamp(-1.0, 1.0).asin(), distance, target }
}

/// The middle and radius of everything drawn (every shown mesh's place), for a camera to frame.
pub fn bounds(world: &mut World) -> Option<(Vec3, f32)> {
    let mut q = world.query::<(&GlobalTransform, &bevy::camera::primitives::Aabb, Option<&InheritedVisibility>)>();
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for (g, a, shown) in q.iter(world) {
        if shown.is_some_and(|s| !s.get()) {
            continue;
        }
        let (c, h) = (Vec3::from(a.center), Vec3::from(a.half_extents));
        for k in 0..8 {
            let corner = c + h * Vec3::new(if k & 1 == 0 { -1.0 } else { 1.0 }, if k & 2 == 0 { -1.0 } else { 1.0 }, if k & 4 == 0 { -1.0 } else { 1.0 });
            let p = g.transform_point(corner);
            (lo, hi) = (lo.min(p), hi.max(p));
        }
    }
    (lo.x <= hi.x).then(|| ((lo + hi) * 0.5, ((hi - lo) * 0.5).length()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_orbit_read_from_a_camera_puts_the_camera_back_where_it_was() {
        let cam = Transform::from_xyz(3.0, 4.0, 5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
        let o = orbit_of(&GlobalTransform::from(cam), Vec3::new(0.0, 1.0, 0.0));
        let back = o.transform();
        assert!(back.translation.distance(cam.translation) < 1e-4, "{:?}", back.translation);
        assert!(back.rotation.angle_between(cam.rotation) < 1e-3);
    }

    #[test]
    fn the_override_holds_the_camera_and_gives_it_back() {
        let mut app = App::new();
        app.init_resource::<FreeView>().add_systems(PostUpdate, apply);
        let own = Transform::from_xyz(0.0, 0.0, 5.0);
        let cam = app.world_mut().spawn((Camera3d::default(), Camera::default(), own)).id();
        app.update();
        let orbit = Orbit { yaw: 1.0, pitch: 0.3, distance: 4.0, target: Vec3::ZERO };
        app.world_mut().resource_mut::<FreeView>().orbit = Some(orbit);
        app.update();
        assert_eq!(*app.world().get::<Transform>(cam).unwrap(), orbit.transform());
        // the composition moves its camera meanwhile: that is where it goes back to
        let moved = Transform::from_xyz(1.0, 2.0, 3.0);
        *app.world_mut().get_mut::<Transform>(cam).unwrap() = moved;
        app.update();
        assert_eq!(*app.world().get::<Transform>(cam).unwrap(), orbit.transform());
        app.world_mut().resource_mut::<FreeView>().orbit = None;
        app.update();
        assert_eq!(*app.world().get::<Transform>(cam).unwrap(), moved);
    }
}
