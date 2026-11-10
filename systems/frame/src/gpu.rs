//! A GPU error (a shader the driver's compiler refuses, memory it hasn't got) is the frame's error, said with what the
//! GPU said: Bevy's own handler asks the app to quit, which no one reads in an engine that steps its app itself, so the
//! frame was read back half drawn and saved as if all was well (#66: DX12's FXC refusing a shader). The engine installs
//! this as Bevy's RenderErrorHandler. Drawing stops either way. Bevy asks again every update while drawing is stopped:
//! only the first is said (a live game's caller prints each one it takes).

use bevy::prelude::*;
use bevy::render::error_handler::{RenderError, RenderErrorPolicy};

use crate::Error;

pub fn gpu_error(e: &RenderError, main: &mut World, _: &mut World) -> RenderErrorPolicy {
    if main.get_resource::<GpuStopped>().is_none() {
        main.insert_resource(GpuStopped);
        let said: Vec<&str> = e.description.lines().map(str::trim).filter(|l| !l.is_empty()).take(4).collect();
        if let Some(mut error) = main.get_resource_mut::<Error>() {
            error.0 = Some(format!("the GPU stopped the render ({:?} error): {}", e.ty, said.join(" ")));
        }
    }
    RenderErrorPolicy::StopRendering
}

/// A frame's error, taken once it's drawn (a render's fed and live-3D paths): said, and the render goes on without what
/// failed (a model that didn't load); or, once the GPU has stopped, the render's error, since no frame from then on has
/// its 3D (an A15 refusing a filtering sampler over a float LUT wrote the page alone and exited 0).
pub fn said(world: &mut World, frame: i64) -> Result<(), String> {
    let err = world.get_resource_mut::<Error>().and_then(|mut e| e.0.take());
    if world.contains_resource::<GpuStopped>() {
        return Err(format!("frame {frame}: the 3D: {}", err.as_deref().unwrap_or("the GPU stopped the render")));
    }
    if let Some(err) = err {
        eprintln!("raea: frame {frame}: the 3D: {err}");
    }
    Ok(())
}

/// The GPU stopped drawing: said once
#[derive(Resource)]
struct GpuStopped;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::error_handler::ErrorType;

    #[test]
    fn a_gpu_error_is_the_frames_error_said_once() {
        let (mut main, mut render) = (World::new(), World::new());
        main.init_resource::<Error>();
        assert!(!main.contains_resource::<GpuStopped>());
        let fxc = RenderError { ty: ErrorType::Validation, description: "Validation Error\n\nCaused by:\n  In Device::create_render_pipeline, label = 'ink_outline'\n    error X3511: unable to unroll loop\n    more".into(), source: None };
        assert!(matches!(gpu_error(&fxc, &mut main, &mut render), RenderErrorPolicy::StopRendering));
        let told = main.resource_mut::<Error>().0.take().expect("the frame's error");
        assert_eq!(told, "the GPU stopped the render (Validation error): Validation Error Caused by: In Device::create_render_pipeline, label = 'ink_outline' error X3511: unable to unroll loop");
        // asked again on the next update: drawing stays stopped, nothing said again
        assert!(matches!(gpu_error(&fxc, &mut main, &mut render), RenderErrorPolicy::StopRendering));
        assert!(main.resource::<Error>().0.is_none());
        assert!(main.contains_resource::<GpuStopped>(), "it stays stopped after its error is taken");
        assert_eq!(said(&mut main, 3), Err("frame 3: the 3D: the GPU stopped the render".into()), "a render fails on");
    }
}
