// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/ui (bevy_ui; drawn by Blitz, blitz-bevy.feature B5): the page.
export { AbsoluteFill, type AbsoluteFillProps } from "@raea/video/timeline";
export * from "@raea/video/region";
// InputRegion (input-region.tsx) joins when it is committed
export * from "@raea/video/panel";
// bevy_ui's Node and Text, and bevy_text's TextSpan, with their values (Val, UiRect, BorderRadius, Overflow)
export { Node, Text, TextSpan, TextShadow, Text2d, UiScale, ImageNode, InlineImage, type TextureSlicer, type SliceScaleMode, ImagePlugin, FontVariationTag, TextureAtlasLayout, type ImageNodeProps, px, percent, vw, vh, vmin, vmax, auto, UiRect, BorderRadius, CornerRadius, Overflow, OverflowClipMargin, type NodeProps, type TextProps, type TextStyleProps, type TextRunProps, type Text2dProps, type Val } from "@raea/render/bevy3d";
