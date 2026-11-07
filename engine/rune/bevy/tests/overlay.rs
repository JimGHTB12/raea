//! A layer that fills a window (RuneOverlay): sized to the window's logical size and scale, and given the window's
//! pointer as Bevy's winit writes it (CursorMoved, MouseButtonInput), so a click on the switch reaches its handler.

use bevy::input::ButtonState;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorMoved, ExitCondition, PrimaryWindow, WindowResolution};
use raea_rune_bevy::{RuneLayerPlugin, RuneOverlay, SpawnRune, Tree, Trees};
use rune::{Cx, El, Env, Runtime, col, el};

#[derive(Default)]
struct State {
    presses: u32,
}

fn view(_: &State, _: &mut Cx<State>) -> El {
    let button = el().key("button").role("button").label("Press").size(80.0, 30.0).fill(rune::tok::accent(0, true)).on_press(|s: &mut State| s.presses += 1);
    col().w_full().h_full().center().child(button)
}

#[test]
fn an_overlay_follows_its_window_and_takes_its_clicks() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() }),
    );
    app.add_plugins(RuneLayerPlugin);
    app.finish();
    app.cleanup();
    // a window of 300 x 200 points at scale 2, as winit would have made it
    let window = app.world_mut().spawn((Window { resolution: WindowResolution::new(600, 400).with_scale_factor_override(2.0), ..default() }, PrimaryWindow)).id();
    let rt = Runtime::new(State::default(), view, (100.0, 100.0), Env::new(true, 1.0));
    let layer = app.world_mut().spawn_rune(Tree::new(rt, 1.0), [0.0; 4]);
    app.world_mut().entity_mut(layer).insert(RuneOverlay::default());
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(app.world().non_send::<Trees>().0[&layer].viewport(), ((300.0, 200.0), 2.0), "the overlay isn't the window's size");
    // the button's centre: the window's middle
    let at = Vec2::new(150.0, 100.0);
    app.world_mut().write_message(CursorMoved { window, position: at, delta: None });
    app.update();
    app.world_mut().write_message(MouseButtonInput { button: MouseButton::Left, state: ButtonState::Pressed, window });
    app.update();
    app.world_mut().write_message(MouseButtonInput { button: MouseButton::Left, state: ButtonState::Released, window });
    for _ in 0..3 {
        app.update();
    }
    let presses = app.world().non_send::<Trees>().0[&layer].state().downcast_ref::<State>().unwrap().presses;
    assert_eq!(presses, 1, "the window's click didn't reach the button");
}
