//! A plain Bevy App drawing a composition (docs/sdk.feature S6a, S8b; docs/blitz-bevy.feature B3 H5): DefaultPlugins as
//! any headless Bevy program sets them, then raea in one RaeaPlugin: raea's plugins (RaeaPlugins, Scene3dPlugin), React
//! (ReactPlugin, running the bundle in this App's World), ScenePlugin, which turns React's <Canvas3D> tree into the
//! scene's rows, and raea-page's PagePlugin and WorldPagePlugin, which draw its HTML layer on Blitz over the scene's
//! camera. No Engine and no second App: the App is the user's own. --scene-only leaves the page out (the 3D alone). The
//! frame is saved as a PNG, to compare with raea's own render on the Blitz page (RAEA_PAGE=blitz).
//!
//! --window draws into a window in place of an image (S6a's window target): a winit window made here, never shown and
//! never focused (no dock icon: macOS's Prohibited activation policy), handed to Bevy as a Window entity with its raw
//! handles, as bevy_winit does, so Bevy's renderer makes its surface and RenderTarget::Window draws there. The PNG is
//! the window's screenshot; it should equal the image target's.
//!
//!   raea bundle EDIT.tsx --composition-bundle -o EDIT.js
//!   cargo run --release -p raea-bridge-bench --example plain_page -- [--scene-only] [--window] EDIT.js OUT.png [WIDTH HEIGHT FRAME FPS]
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{ExitCondition, RawHandleWrapper, WindowRef, WindowResolution, WindowWrapper};
use raea_runtime_page::RaeaPlugin;
use std::path::PathBuf;

struct Args {
    js: PathBuf,
    out: String,
    size: (u32, u32),
    frame: i64,
    fps: f64,
    scene_only: bool,
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut flag = |f: &str| args.iter().position(|a| a == f).map(|i| args.remove(i)).is_some();
    let (scene_only, window) = (flag("--scene-only"), flag("--window"));
    let (Some(js), Some(out)) = (args.first(), args.get(1)) else {
        eprintln!("usage: plain_page [--scene-only] [--window] EDIT.js OUT.png [WIDTH HEIGHT FRAME FPS]");
        std::process::exit(2);
    };
    let n = |i: usize, d: u32| args.get(i).and_then(|a| a.parse().ok()).unwrap_or(d);
    let a = Args { js: PathBuf::from(js).canonicalize().expect("the bundle"), out: out.clone(), size: (n(2, 640), n(3, 360)), frame: n(4, 30) as i64, fps: n(5, 30) as f64, scene_only };
    if window {
        std::process::exit(in_window(a));
    }
    let mut app = app();
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(a.size.0, a.size.1, TextureFormat::Rgba8UnormSrgb, None));
    std::process::exit(shoot(app, &a, RenderTarget::Image(image.clone().into()), Screenshot::image(image)));
}

/// DefaultPlugins as a headless Bevy program sets them: no window of Bevy's making (--window hands it one).
fn app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    );
    app
}

/// raea added to the App drawing into `target`, then the frame saved by `shot`: 0, or 1 with what went wrong.
fn shoot(mut app: App, a: &Args, target: RenderTarget, shot: Screenshot) -> i32 {
    // raea in one add_plugins (docs/sdk.feature S8b): its Engine plugins, React, the scene and (unless scene_only) the page
    app.add_plugins(RaeaPlugin { frame: Some(a.frame), fps: a.fps, scene_only: a.scene_only, ..RaeaPlugin::new(a.js.clone(), target, a.size) });
    app.finish();
    app.cleanup();

    // React commits on its own thread: update until the scene's camera is in the World, then a few frames more so
    // its pipelines and shadows are made, then one screenshot of the target
    let mut seen = None;
    for i in 0..600 {
        app.update();
        // the scene's camera, or with the page, the page drawn over it (or over a camera of its own, the 3D in its
        // layered slot)
        let cams = if a.scene_only {
            app.world_mut().query::<&Camera3d>().iter(app.world()).count()
        } else {
            app.world_mut().query_filtered::<(), (With<Camera>, With<raea_runtime_twins::overlay::PageOverlay>)>().iter(app.world()).count()
        };
        if cams > 0 && seen.is_none() {
            seen = Some(i);
        }
        if seen.is_some_and(|s| i >= s + 8) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    if seen.is_none() {
        let kinds: std::collections::BTreeSet<String> = app.world_mut().query::<&raea_runtime_react::ReactKind>().iter(app.world()).map(|k| k.0.clone()).collect();
        let ents = app.world().resource::<raea_runtime_react::ReactEntities>();
        eprintln!("plain_page: no 3D camera after 600 updates; React applied {} commits ({} ops, {} nodes), kinds {:?}", ents.commits, ents.ops, ents.by_id.len(), kinds);
        return 1;
    }
    app.world_mut().spawn(shot).observe(save_to_disk(a.out.clone()));
    for _ in 0..60 {
        app.update();
        if std::path::Path::new(&a.out).exists() {
            println!("plain_page: {}", a.out);
            return 0;
        }
    }
    eprintln!("plain_page: no frame saved");
    1
}

/// --window: the frame drawn into a hidden window of winit's, which this program's event loop makes (macOS wants
/// windows made on the main thread, inside the loop).
fn in_window(a: Args) -> i32 {
    use winit::application::ApplicationHandler;
    use winit::event_loop::{ActiveEventLoop, EventLoop};

    struct Run {
        a: Args,
        code: i32,
    }
    impl ApplicationHandler for Run {
        fn resumed(&mut self, el: &ActiveEventLoop) {
            let (w, h) = self.a.size;
            let attrs = winit::window::Window::default_attributes()
                .with_title("plain_page")
                .with_visible(false)
                .with_active(false)
                .with_inner_size(winit::dpi::PhysicalSize::new(w, h));
            self.code = match el.create_window(attrs).map(WindowWrapper::new) {
                Ok(win) => match RawHandleWrapper::new(&win) {
                    Ok(handles) => {
                        let mut app = app();
                        // the window at the composition's size in pixels (scale 1: a pixel a pixel, as the image's)
                        let window = Window { resolution: WindowResolution::new(w, h).with_scale_factor_override(1.0), visible: false, focused: false, ..default() };
                        let e = app.world_mut().spawn((window, handles)).id();
                        let code = shoot(app, &self.a, RenderTarget::Window(WindowRef::Entity(e)), Screenshot::window(e));
                        drop(win);
                        code
                    }
                    Err(e) => {
                        eprintln!("plain_page: the window's handles: {e}");
                        1
                    }
                },
                Err(e) => {
                    eprintln!("plain_page: no window: {e}");
                    1
                }
            };
            el.exit();
        }
        // the window is never shown, so nothing asks to close it; if the system does, the run ends unsaved
        fn window_event(&mut self, el: &ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
            if matches!(event, winit::event::WindowEvent::CloseRequested) {
                eprintln!("plain_page: the window was closed before the frame was saved");
                el.exit();
            }
        }
    }

    let mut builder = EventLoop::builder();
    // no dock icon and no focus taken from whatever is in front (the window is never shown either)
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder.with_activation_policy(ActivationPolicy::Prohibited).with_activate_ignoring_other_apps(false);
    }
    let el = match builder.build() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("plain_page: no event loop: {e}");
            return 1;
        }
    };
    let mut run = Run { a, code: 1 };
    if let Err(e) = el.run_app(&mut run) {
        eprintln!("plain_page: the event loop: {e}");
        return 1;
    }
    run.code
}
