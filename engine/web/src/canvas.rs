//! The page's canvas as a Bevy window (no winit: a window entity whose handle is the canvas, which wgpu's WebGPU
//! backend makes its surface from), and a camera on it that shows the engine's target, nearest-neighbour: the scene is
//! drawn at its own size (a pixel look's size) and the canvas scales it up with hard pixels.

use std::ptr::NonNull;

use bevy::camera::{ClearColorConfig, RenderTarget};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::window::{RawHandleWrapper, WindowRef, WindowResolution, WindowWrapper};
use raea_render::engine::Engine;
use raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, WebCanvasWindowHandle, WebDisplayHandle, WindowHandle};
use wasm_bindgen::JsValue;

/// The canvas, held where its handle points (a boxed JsValue, as raw-window-handle's WebCanvas wants).
struct Canvas(Box<JsValue>);

// SAFETY: wasm32 here is one thread; the canvas is only touched from it
unsafe impl Send for Canvas {}
unsafe impl Sync for Canvas {}

impl HasWindowHandle for Canvas {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let obj = NonNull::from(&*self.0).cast();
        // SAFETY: the handle points at the JsValue this Canvas owns for as long as the window entity holds it
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::WebCanvas(WebCanvasWindowHandle::new(obj))) })
    }
}

impl HasDisplayHandle for Canvas {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // SAFETY: the web's display handle holds nothing
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Web(WebDisplayHandle::new())) })
    }
}

/// The canvas as a window of the App (before it's finished, so the renderer sees it from the first update).
pub fn attach(world: &mut World, canvas: &web_sys::HtmlCanvasElement) -> Entity {
    let (w, h) = (canvas.width().max(1), canvas.height().max(1));
    let wrapper = WindowWrapper::new(Canvas(Box::new(JsValue::from(canvas.clone()))));
    let handle = RawHandleWrapper::new(&wrapper).expect("a canvas always has a handle");
    world
        .spawn((
            Window { resolution: WindowResolution::new(w, h).with_scale_factor_override(1.0), present_mode: bevy::window::PresentMode::AutoVsync, ..default() },
            handle,
        ))
        .id()
}

/// The camera on the canvas, and the target on it filling the canvas, sampled nearest (the tag row under it cut off).
pub fn show(e: &mut Engine, window: Entity) {
    let (w, h) = e.size();
    let target = e.target_image();
    let world = e.world_mut();
    if let Some(mut img) = world.resource_mut::<Assets<Image>>().get_mut(&target) {
        img.sampler = ImageSampler::nearest();
    }
    let cam = world
        .spawn((
            Camera2d,
            // after every camera that draws into the target
            Camera { order: 10_000, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() },
            RenderTarget::Window(WindowRef::Entity(window)),
        ))
        .id();
    world.spawn((
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ImageNode { image: target, rect: Some(Rect::new(0.0, 0.0, w as f32, h as f32)), ..default() },
        UiTargetCamera(cam),
    ));
}

/// The canvas's pixel size changed.
pub fn resize(world: &mut World, window: Entity, w: u32, h: u32) {
    if let Some(mut win) = world.get_mut::<Window>(window) {
        win.resolution.set_physical_resolution(w.max(1), h.max(1));
    }
}
