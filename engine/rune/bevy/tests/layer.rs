//! A Rune layer in a headless Bevy app: its Image holds, to the byte, what Rune's own offscreen renderer draws of the
//! same tree at the same frame; a press sent to it reaches the tree's handler; and Bevy samples it in the frame it was
//! drawn (a lit scene with the Image on a quad reads the layer's frame counter, not the last frame's).

use bevy::prelude::*;
use bevy::window::ExitCondition;
use raea_rune_bevy::{Input, RuneLayer, RuneLayerPlugin, SpawnRune, Tree, Trees};
use rune::{Cx, El, Env, Runtime, col, el, text};

#[derive(Default)]
struct State {
    on: bool,
    presses: u32,
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let t = rune::tok::theme(true);
    let knob = el().key("knob").size(18.0, 18.0).radius(9.0).fill(t.knob).absolute(if s.on { 18.0 } else { 2.0 }, 2.0);
    let switch = el()
        .key("switch")
        .role("switch")
        .label("Lights")
        .checked(s.on)
        .size(38.0, 22.0)
        .radius(11.0)
        .fill(if s.on { rune::tok::accent(0, true) } else { t.control_off })
        .motion(rune::Spring::SNAPPY)
        .on_press(|s: &mut State| {
            s.on = !s.on;
            s.presses += 1;
        })
        .child(knob);
    col().w_full().h_full().center().gap(14.0).fill(t.bg).child(text(if s.on { "Lights on" } else { "Lights off" }).color(t.text)).child(switch)
}

const PT: (f32, f32) = (240.0, 160.0);
const SCALE: f32 = 2.0;
const CLEAR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
const STEP: f64 = 1.0 / 60.0;

fn runtime() -> Runtime<State> {
    Runtime::new(State::default(), view, PT, Env::new(true, SCALE))
}

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build() // no multi_threaded feature here, so no pipelined rendering to disable
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() }),
    );
    app.add_plugins(RuneLayerPlugin);
    app.finish();
    app.cleanup();
    let e = app.world_mut().spawn_rune(Tree::new(runtime(), SCALE), CLEAR);
    app.world_mut().get_mut::<RuneLayer>(e).unwrap().step = Some(STEP);
    (app, e)
}

/// The layer's Image read back, rows tight.
fn read(app: &App, e: Entity) -> Vec<u8> {
    let handle = app.world().get::<RuneLayer>(e).unwrap().image.clone();
    raea_rune_bevy::readback::read(app, &handle).expect("the layer's GpuImage")
}

/// The same tree, the same frames, drawn by Rune's own headless GPU and offscreen target.
fn rune_own(frames: u32) -> Vec<u8> {
    use rune::render::{Gpu, Offscreen, Renderer};
    let mut tree = Tree::new(runtime(), SCALE);
    for _ in 0..frames {
        raea_rune_bevy::AnyTree::step(&mut tree, STEP);
    }
    let gpu = Gpu::headless();
    let (w, h) = raea_rune_bevy::AnyTree::pixels(&tree);
    let target = Offscreen::with_format(&gpu.device, w, h, raea_rune_bevy::FORMAT);
    let mut renderer = Renderer::new(&gpu.device, raea_rune_bevy::FORMAT);
    renderer.upload_images(&gpu.device, &gpu.queue, &mut tree.rt.images);
    renderer.prepare(&gpu.device, &gpu.queue, &mut tree.rt.text.atlas, &tree.rt.instances, (w, h));
    renderer.prepare_surfaces(&gpu.device, &gpu.queue, &tree.rt.surfaces, (w, h));
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    renderer.draw(&mut enc, &target.view, CLEAR);
    target.copy(&mut enc);
    renderer.submit(&gpu.queue, enc);
    target.read(&gpu.device)
}

#[test]
fn layer_matches_runes_own_render_and_takes_a_press() {
    let (mut app, e) = app();
    // the first update puts the Image on the GPU; the layer draws from the second
    for _ in 0..12 {
        app.update();
    }
    let ours = read(&app, e);
    let own = rune_own(12);
    assert_eq!(ours.len(), (PT.0 * SCALE * PT.1 * SCALE * 4.0) as usize);
    assert!(ours.chunks(4).any(|p| p[3] > 0), "the layer drew nothing");
    let diff = ours.iter().zip(&own).filter(|(a, b)| a != b).count();
    assert_eq!(diff, 0, "{diff} bytes differ from Rune's own render of the same frame");

    // a press on the switch's centre, where Rune laid it out: the layer's points, as a window's overlay sends them
    let switch = {
        let mut probe = runtime();
        probe.frame(0.0);
        probe.find("switch").expect("the switch laid out")
    };
    {
        let mut trees = app.world_mut().non_send_mut::<Trees>();
        let t = trees.0.get_mut(&e).unwrap();
        t.input(Input::Move(switch.0, switch.1));
        t.input(Input::Down(switch.0, switch.1));
        t.input(Input::Up(switch.0, switch.1));
    }
    for _ in 0..30 {
        app.update();
    }
    let trees = app.world().non_send::<Trees>();
    let s = trees.0.get(&e).unwrap().state().downcast_ref::<State>().unwrap();
    assert_eq!((s.presses, s.on), (1, true), "the press didn't reach the switch at {switch:?}");
    assert_ne!(read(&app, e), ours, "the layer still shows the switch off");
}
