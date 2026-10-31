use anyrender::{Filter, PaintScene};
use kurbo::{Affine, Shape};
use peniko::{BlendMode, Fill, Mix};
use std::{cell::Cell, sync::Arc};

const LAYER_LIMIT: u32 = 1024;

#[derive(Default)]
pub(crate) struct LayerManager {
    layers_used: Cell<u32>,
    layer_depth: Cell<u32>,
    layers_wanted: Cell<u32>,

    #[allow(unused)] // Only used for debugging. Enabled as required.
    layer_depth_used: Cell<u32>,
}

impl LayerManager {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn maybe_with_layer<S: PaintScene, F: FnOnce(&mut S)>(
        &self,
        scene: &mut S,
        condition: bool,
        opacity: f32,
        transform: Affine,
        shape: &impl Shape,
        filter: Option<Arc<Filter>>,
        backdrop_filter: Option<Arc<Filter>>,
        paint_layer: F,
    ) {
        self.maybe_with_layer_filled(scene, condition, Fill::NonZero, opacity, transform, shape, filter, backdrop_filter, paint_layer);
    }

    /// raea: maybe_with_layer composited onto what's under it by `blend` (mix-blend-mode), always a group of its own
    /// when there is one (isolation: isolate is Some(normal))
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn maybe_with_blend_layer<S: PaintScene, F: FnOnce(&mut S)>(
        &self,
        scene: &mut S,
        condition: bool,
        opacity: f32,
        blend: Option<BlendMode>,
        transform: Affine,
        shape: &impl Shape,
        filter: Option<Arc<Filter>>,
        backdrop_filter: Option<Arc<Filter>>,
        paint_layer: F,
    ) {
        let layer_used = self.push(scene, condition, Fill::NonZero, opacity, blend, transform, shape, filter, backdrop_filter);
        paint_layer(scene);
        self.maybe_pop_layer(scene, layer_used);
    }

    /// raea: maybe_with_layer with the shape filled by `fill`'s rule (a clip-path's evenodd)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn maybe_with_layer_filled<S: PaintScene, F: FnOnce(&mut S)>(
        &self,
        scene: &mut S,
        condition: bool,
        fill: Fill,
        opacity: f32,
        transform: Affine,
        shape: &impl Shape,
        filter: Option<Arc<Filter>>,
        backdrop_filter: Option<Arc<Filter>>,
        paint_layer: F,
    ) {
        let layer_used = self.push(scene, condition, fill, opacity, None, transform, shape, filter, backdrop_filter);
        paint_layer(scene);
        self.maybe_pop_layer(scene, layer_used);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn maybe_push_layer(
        &self,
        scene: &mut impl PaintScene,
        condition: bool,
        opacity: f32,
        transform: Affine,
        shape: &impl Shape,
        filter: Option<Arc<Filter>>,
        backdrop_filter: Option<Arc<Filter>>,
    ) -> bool {
        self.push(scene, condition, Fill::NonZero, opacity, None, transform, shape, filter, backdrop_filter)
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &self,
        scene: &mut impl PaintScene,
        condition: bool,
        fill: Fill,
        opacity: f32,
        blend: Option<BlendMode>,
        transform: Affine,
        shape: &impl Shape,
        filter: Option<Arc<Filter>>,
        backdrop_filter: Option<Arc<Filter>>,
    ) -> bool {
        if !condition {
            return false;
        }
        self.layers_wanted.update(|x| x + 1);

        // Check if clips are above limit
        let layers_available = self.layers_used.get() <= LAYER_LIMIT;
        if !layers_available {
            return false;
        }

        // Actually push the layer
        if opacity == 1.0 && filter.is_none() && backdrop_filter.is_none() && blend.is_none() {
            scene.push_clip_layer(fill, transform, shape);
        } else {
            scene.push_layer(
                fill,
                blend.unwrap_or(Mix::Normal.into()),
                opacity,
                transform,
                shape,
                filter,
                backdrop_filter,
            );
        };

        // Update accounting
        self.layers_used.update(|x| x + 1);
        self.layer_depth.update(|x| x + 1);
        self.layer_depth.update(|x| x.max(self.layer_depth.get()));

        true
    }

    pub(crate) fn maybe_pop_layer(&self, scene: &mut impl PaintScene, condition: bool) {
        if condition {
            scene.pop_layer();
            self.layer_depth.update(|x| x - 1);
        }
    }
}
