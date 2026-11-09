//! A shader raea made for a scene (`register`: a ShaderExtension's surface and lit hooks, an own material's, a UI
//! material's) that didn't compile. Bevy logs the compiler's error, leaves the pipeline out and draws on without it, so
//! a render came out without the shader (black, for a ShaderExtension) and still said APPLIED. Each frame the render
//! world looks for those pipelines and refuses the render with the compiler's error (raea-gpu-device `refuse`, which
//! the render reads back with the frame).
use bevy::prelude::*;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache, PipelineDescriptor};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::shader::{Shader, ShaderCacheError};
use std::collections::HashSet;
use std::sync::Mutex;

/// The uuids `register` gave the shaders it added.
static OURS: Mutex<Option<HashSet<u128>>> = Mutex::new(None);

/// A shader `register` added: its errors refuse the render.
pub(crate) fn ours(id: u128) {
    OURS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashSet::new).insert(id);
}

pub(crate) fn install(app: &mut App) {
    if let Some(r) = app.get_sub_app_mut(RenderApp) {
        r.add_systems(Render, check.in_set(RenderSystems::Cleanup));
    }
}

/// The shaders a pipeline runs.
fn shaders(d: &PipelineDescriptor) -> Vec<AssetId<Shader>> {
    match d {
        PipelineDescriptor::RenderPipelineDescriptor(d) => std::iter::once(d.vertex.shader.id()).chain(d.fragment.as_ref().map(|f| f.shader.id())).collect(),
        PipelineDescriptor::ComputePipelineDescriptor(d) => vec![d.shader.id()],
        PipelineDescriptor::MeshPipelineDescriptor(d) => d.fragment.as_ref().map(|f| f.shader.id()).into_iter().collect(),
    }
}

fn check(cache: Res<PipelineCache>, mut told: Local<HashSet<u128>>) {
    let ours = OURS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(ours) = ours.as_ref() else { return };
    for p in cache.pipelines() {
        let CachedPipelineState::Err(ShaderCacheError::ProcessShaderError(why) | ShaderCacheError::CreateShaderModule(why)) = &p.state else { continue };
        for id in shaders(&p.descriptor) {
            let AssetId::Uuid { uuid } = id else { continue };
            let u = uuid.as_u128();
            if ours.contains(&u) && told.insert(u) {
                raea_gpu_device::check::refuse(message(why));
            }
        }
    }
}

/// The refusal: the compiler's error without its terminal colours. Its line numbers count in the module raea wraps the
/// source in, so it quotes the line itself.
fn message(why: &str) -> String {
    let mut plain = String::with_capacity(why.len());
    let mut chars = why.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // an escape sequence: up to and including its final letter
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            plain.push(c);
        }
    }
    format!("a shader in the scene didn't compile, so it would have drawn without it:\n{}", plain.trim_end())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_message_loses_its_colours() {
        let m = super::message("\u{1b}[31merror\u{1b}[0m: use of a reserved word `patch`\n");
        assert!(m.ends_with("error: use of a reserved word `patch`"), "{m}");
    }
}
