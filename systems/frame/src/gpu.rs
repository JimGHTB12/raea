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
        let fxc = RenderError { ty: ErrorType::Validation, description: "Validation Error\n\nCaused by:\n  In Device::create_render_pipeline, label = 'ink_outline'\n    error X3511: unable to unroll loop\n    more".into(), source: None };
        assert!(matches!(gpu_error(&fxc, &mut main, &mut render), RenderErrorPolicy::StopRendering));
        let said = main.resource_mut::<Error>().0.take().expect("the frame's error");
        assert_eq!(said, "the GPU stopped the render (Validation error): Validation Error Caused by: In Device::create_render_pipeline, label = 'ink_outline' error X3511: unable to unroll loop");
        // asked again on the next update: drawing stays stopped, nothing said again
        assert!(matches!(gpu_error(&fxc, &mut main, &mut render), RenderErrorPolicy::StopRendering));
        assert!(main.resource::<Error>().0.is_none());
    }
}
