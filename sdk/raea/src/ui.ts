// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/ui (bevy_ui; drawn by Blitz, blitz-bevy.feature B5): the page.
export { AbsoluteFill, type AbsoluteFillProps } from "@raea/video/timeline";
export * from "@raea/video/region";
// InputRegion (input-region.tsx) joins when it is committed
export * from "@raea/video/panel";
// bevy_ui's Node and Text, and bevy_text's TextSpan, with their values (Val, UiRect, BorderRadius, Overflow)
export { Node, GhostNode, Text, TextSpan, TextShadow, Text2d, UiScale, Viewport, UiTargetCamera, UiTexture, targetImage, ImageNode, InlineImage, type TextureSlicer, type SliceScaleMode, ImagePlugin, FontVariationTag, TextureAtlasLayout, type ImageNodeProps, px, percent, vw, vh, vmin, vmax, auto, UiRect, BorderRadius, CornerRadius, Overflow, OverflowClipMargin, type NodeProps, type TextProps, type TextStyleProps, type TextRunProps, type Text2dProps, type Val } from "@raea/render/bevy3d";
export { LinearGradient, RadialGradient, ConicGradient, ColorStop, AngularColorStop, UiPosition, type Gradient, type RadialGradientShape, type InterpolationColorSpace } from "@raea/render/bevy3d";
export { GridTrack, RepeatedGridTrack, GridPlacement, type GridTrackRepetition, type GridAutoFlow, type MinTrackSizingFunction, type MaxTrackSizingFunction, FontSource, type GenericFontFamily } from "@raea/render/bevy3d";
export { Rot2, Val2, UiTransform, BoxShadow, type ShadowStyle } from "@raea/render/bevy3d";
export { FontFeatureTag, FontWidth } from "@raea/render/bevy3d";
export { GlobalUiDebugOptions, type UiDebugOptions } from "@raea/render/bevy3d";
export type { IgnoreScroll } from "@raea/render/bevy3d";
export { Scrollbar, ScrollbarThumb, type ScrollbarProps, type ScrollbarThumbProps, type ControlOrientation } from "@raea/render/bevy3d";
// bevy_text's EditableText on a Node, its caret drawn while it has focus (docs/control.feature FC0a)
export { EditableText, type EditableTextProps, type TextCursorStyle } from "@raea/render/bevy3d";
