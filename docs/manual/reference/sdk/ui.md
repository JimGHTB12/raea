---
title: "raea/ui"
summary: "bevy_ui: Node and its layout fields, Text, images, scrolling, gradients and units."
order: 83
sources: ["sdk/raea/src/ui.ts", "sdk/render/src/bevy3d/ui.tsx"]
---

`raea/ui` is bevy_ui in TSX. `<Node>` takes Bevy's layout fields as props (`width`, `flex_direction`,
`justify_content`, `padding` and the rest, with Bevy's defaults), and `px`, `percent`, `vw` and the other units build
Bevy's `Val`. `<Text>` and `<TextSpan>` are Bevy's text; `<Text2d>` is text in the 2D world. Plain HTML elements with
CSS work too; these components are for matching Bevy exactly.

```tsx
import { staticFile } from "raea";
import { Srgba } from "raea/color";
import { Node, Text, UiRect, percent, px } from "raea/ui";

const Panel = () => (
  <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center" flex_direction="Column">
    <Node width={px(250)} height={px(250)} margin={UiRect.bottom(px(15))} background_color={new Srgba(0.92, 0.14, 0.05)} />
    <Text font={staticFile("fonts/FiraSans-Bold.ttf")} font_size={{ Px: 33 }} color={new Srgba(0.9, 0.9, 0.9)}>Hello</Text>
  </Node>
);
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { AbsoluteFill, ACTION_KEY, actionsIn, AngularColorStop } from "raea/ui"
```

`raea/ui` exports 20 components, 1 hook, 12 functions, 30 values, 37 types. Bevy's counterpart: `bevy_ui; drawn by Blitz, blitz-bevy.feature B5`. Source: [sdk/raea/src/ui.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/ui.ts).

## Components

### `AbsoluteFill`

A box over the whole of its parent, laid out as a flex column; `style` is merged over that.

| Prop | Type | What it does |
| --- | --- | --- |
| `style` | `CSSProperties` | merged over the fill's own style (absolute, inset 0, a flex column) |
| `className` | `string` | Tailwind or CSS classes |
| `children` | `ReactNode` | what fills it |

### `BoxShadowSamples`

bevy_ui_render's BoxShadowSamples: the steps each shadow's blur is summed in down its height (4 by default). Bevy sets
it on a UI camera; here it wraps the UI that camera draws.

Bevy: `bevy_ui_render::BoxShadowSamples`.

| Prop | Type | What it does |
| --- | --- | --- |
| `samples` (required) | `number` | steps each shadow's blur is summed in (4) |
| `children` | `ReactNode` | the UI it applies to |

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx).

### `EditableText`

A text input's text, its caret drawn while it has focus.

Bevy: `bevy_text::EditableText`.

| Prop | Type | What it does |
| --- | --- | --- |
| `children` | `string` | the text it starts with (EditableText::new) |
| `visible_lines` | `number \| null` | visible_lines: its height in lines (1 by default; null leaves the height to the layout) |
| `visible_width` | `number \| null` | visible_width: its width in "0"s (None by default: the width it's given) |
| `allow_newlines` | `boolean` | allow_newlines (false by default): Enter adds a line, under input |
| `max_characters` | `number \| null` | max_characters (None by default): the most characters typing can make it hold, under input |
| `filter` | `(c: string) => boolean` | EditableTextFilter: the characters typing may add (each tested alone), under input |
| `read_write_mode` | `"Editable" \| "ReadOnly" \| "Static"` | TextReadWriteMode ("Editable" by default): ReadOnly takes no edits, under input; Static draws no caret either |
| `cursor_width` | `number` | cursor_width: the caret's width as a share of the font size (0.2 by default) |
| `cursor_blink_period` | `number` | cursor_blink_period in seconds (1 by default) |
| `cursor_style` | `TextCursorStyle` | TextCursorStyle |
| `auto_focus` | `boolean` | AutoFocus: it has the input focus, so its caret is drawn |
| `justify` | `TextProps["justify"]` | TextLayout's justify and linebreak |
| `linebreak` | `TextProps["linebreak"]` | where lines break: WordBoundary, AnyCharacter, WordOrCharacter or NoWrap |

Also takes the props of [`NodeProps`](#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`); [`TextStyleProps`](#textstyleprops).

Used in: [ui/text/multiline_text_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiline_text_input.tsx), [ui/text/multiple_text_inputs](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiple_text_inputs.tsx), [ui/text/editable_text_filter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/editable_text_filter.tsx), [ui/text/ime_support](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/ime_support.tsx), [ui/text/text_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text_input.tsx), [ui/widgets/virtual_keyboard](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/virtual_keyboard.tsx).

### `GhostNode`

bevy_ui's GhostNode: no box of its own, its children laid out as its parent's (CSS display: contents), each one a
root when it is at the top.

Bevy: `bevy_ui::experimental::GhostNode`.

| Prop | Type | What it does |
| --- | --- | --- |
| `children` | `ReactNode` | what's laid out as the parent's children |

Used in: [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx).

### `GlobalUiDebugOptions`

bevy_ui's GlobalUiDebugOptions: the debug overlay for every node inside that has no UiDebugOptions of its own
(a node's `ui_debug`).

Bevy: `bevy_ui_render::GlobalUiDebugOptions`.

| Prop | Type | What it does |
| --- | --- | --- |
| `children` | `ReactNode` | the UI the overlay draws over |

Also takes the props of [`UiDebugOptions`](#uidebugoptions).

Used in: [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx).

### `ImageNode`

bevy_ui's ImageNode: a Node showing a picture stretched over its content box (inside its padding and border),
sized by the picture when its width or height is auto, as bevy_ui's ImageNodeSize measures it. No children (an
<img>).

Bevy: `bevy_ui::widget::image::ImageNode`.

| Prop | Type | What it does |
| --- | --- | --- |
| `image` (required) | `string \| { path: string; sampler: SamplerSpec }` | the picture: staticFile("branding/icon.png"), or one with its own sampler ({ path, sampler: { filter: "nearest" } }, Bevy's load_with_settings) |
| `color` | `Srgba` | its tint: white (none) by default; only white or a grey multiplies here yet (a picture's brightness) |
| `flip_x` | `boolean` | Bevy: Whether the image should be flipped along its x-axis. |
| `flip_y` | `boolean` | Bevy: Whether the image should be flipped along its y-axis. |
| `texture_atlas` | `{ layout: TextureAtlasLayout; index: number }` | TextureAtlas: the one cell of a sheet laid out by a TextureAtlasLayout to draw (the picture is the sheet) |
| `image_mode` | `"Auto" \| "Stretch" \| { Sliced: TextureSlicer }` | NodeImageMode: "Auto" (the default) sizes the node by the picture and draws it contained in the content box, its aspect kept; "Stretch" gives the node no size of its own and stretches the picture over its content box; Sliced cuts the picture in nine by a TextureSlicer (its corners kept, its sides and centre stretched or tiled), drawn as CSS border-image with bevy_ui's scale; a sliced ImageNode holds children |

Also takes the props of [`NodeProps`](#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/images/image_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node.tsx), [ui/images/ui_texture_slice_flip_and_tile](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice_flip_and_tile.tsx), [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice.tsx), [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx), [ui/ui_transform](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_transform.tsx), [ui/scroll_and_overflow/overflow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow.tsx), and 5 more.

### `ImagePlugin`

bevy_image's ImagePlugin: its default_sampler for every picture drawn inside it (Linear by default;
ImagePlugin::default_nearest() is default_sampler="Nearest", for pixel art).

Bevy: `bevy_image::ImagePlugin`.

| Prop | Type | What it does |
| --- | --- | --- |
| `default_sampler` | `"Linear" \| "Nearest"` | "Linear" (the default) or "Nearest" (pixel art) |
| `children` | `ReactNode` | the UI and sprites whose pictures it samples |

Used in: [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx).

### `InlineImage`

bevy_ui's InlineImage: a picture in a <Text>'s line, as an inline box on its baseline (bevy_text's parley lays it
out so, and Blitz's parley here), `width` and `height` px (the other by its aspect, or its own size), tinted by
`color` (each pixel times it: the picture multiplied over the tint, its own alpha the mask).

Bevy: `bevy_ui::widget::InlineImage`.

| Prop | Type | What it does |
| --- | --- | --- |
| `image` (required) | `string` | Bevy: Handle to the texture. |
| `color` | `Color` | Bevy: The tint color used to draw the image. |
| `width` | `number` | Bevy: Set a fixed width in logical pixels for the image's inline box target. |
| `height` | `number` | Bevy: Set a fixed height in logical pixels for the image's inline box target. |

Used in: [ui/text/inline_image](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/inline_image.tsx).

### `MaterialNode`

A node its own fragment shader draws (MaterialNode(handle) with a UiMaterial asset in Bevy).

Bevy: `bevy_ui_render::ui_material::MaterialNode`.

| Prop | Type | What it does |
| --- | --- | --- |
| `fragment` (required) | `string` | the fragment shader's file, as UiMaterial::fragment_shader names one: staticFile("shaders/x.wesl") |
| `uniforms` | `Record<string, Uniform \| Record<string, Uniform>>` | each `var<uniform>` the shader declares, by name: a number, [x, y], [x, y, z], [x, y, z, w] (a LinearRgba's to_f32_array()) or "#rrggbb"; a struct's as {field: value} |
| `textures` | `Record<string, string>` | each `texture_2d<f32>` it declares, by name: a picture file (staticFile), sampled with Bevy's default sampler |
| `defs` | `Record<string, boolean \| number>` | shader defs, as a Material's |
| `children` | `ReactNode` | the nodes inside it |

Also takes the props of [`NodeProps`](#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/ui_material](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_material.tsx).

### `Node`

A box of Bevy's UI, laid out as bevy_ui lays it (taffy, with Bevy's defaults). Put it over the 3D in an
<AbsoluteFill> (after the <Canvas3D>): each root fills the frame as Bevy's window.

Bevy: `bevy_ui::ui_node::Node`.

Takes [`NodeProps`](#nodeprops)'s props.

Used in: [ui/images/image_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node.tsx), [ui/images/ui_texture_slice_flip_and_tile](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice_flip_and_tile.tsx), [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice.tsx), [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx), and 56 more.

### `Region`

A box at the band its name stands for, clear of the face perception found at this frame.

| Prop | Type | What it does |
| --- | --- | --- |
| `name` | `RegionName` | the band: "auto" (default) takes the larger of top and bottom; top and bottom stay clear of the face; center is the middle third; full is the safe area |
| `margin` | `number` | the gap kept from the face and the sides, as a share of the width (0.06) |
| `style` | `CSSProperties` | merged under the box's own placement |
| `className` | `string` | Tailwind or CSS classes |
| `children` | `ReactNode` | what goes in the band; `useRegion()` reads its box |

### `Scrollbar`

A scrollbar's track: its ScrollbarThumb inside shows how much of its target is in view and where.

Bevy: `bevy_ui_widgets::Scrollbar`.

| Prop | Type | What it does |
| --- | --- | --- |
| `target` (required) | `string` | the Name of the Node it scrolls (an overflow Scroll one) |
| `orientation` (required) | `ControlOrientation` | Bevy: Whether the scrollbar is vertical or horizontal. |
| `min_thumb_length` (required) | `number` | the thumb's least length, px |
| `children` | `ReactNode` | its ScrollbarThumb |

Also takes the props of [`NodeProps`](#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/scroll_and_overflow/scrollbars](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scrollbars.tsx).

### `ScrollbarThumb`

A Scrollbar's thumb: the track's width across it, its length the share of the target in view.

Bevy: `bevy_ui_widgets::ScrollbarThumb`.

| Prop | Type | What it does |
| --- | --- | --- |
| `border_radius` | `BorderRadius` | Bevy: Border radius of the scrollbar thumb, used to update `ComputedNode::border_radius` in `UiSystems::Layout`. |
| `border` | `UiRect` | the border's width on each side |
| `background_color` | `NodeProps["background_color"]` | BackgroundColor |
| `border_color` | `NodeProps["background_color"]` | BorderColor, every side |

Used in: [ui/scroll_and_overflow/scrollbars](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scrollbars.tsx).

### `Text`

Bevy's UI text: a Node whose content is its string and its <TextSpan> children, in its TextFont, TextColor,
TextLayout and LineHeight (Bevy's defaults when left out).

Bevy: `bevy_ui::widget::text::Text`.

Takes [`TextProps`](#textprops)'s props.

Used in: [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice.tsx), [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx), [ui/layout/flex_layout](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/flex_layout.tsx), [ui/layout/size_constraints](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/size_constraints.tsx), and 47 more.

### `Text2d`

Text in Bevy's 2D world, its block's centre at its Transform (Text2d; Bevy's Anchor::CENTER unless `anchor` says). bevy_text
lays it out as a <Text>'s. Inside a <Canvas3D> it is Bevy's own Text2d (text2d.tsx), ordered with the 2D parts by z;
after it, in the <AbsoluteFill>, it is that Text placed on the frame, over the 2D and 3D parts.

Bevy: `bevy_sprite::text2d::Text2d`.

| Prop | Type | What it does |
| --- | --- | --- |
| `transform` | `TransformValue` | its place in Bevy's 2D world: the frame's centre the origin, +Y up, a unit a pixel, turns counterclockwise |
| `justify` | `"Left" \| "Center" \| "Right" \| "Justified" \| "Start" \| "End"` | how its lines line up with each other (TextLayout's Justify, Left by default) |
| `linebreak` | `"WordBoundary" \| "AnyCharacter" \| "WordOrCharacter" \| "NoWrap"` | TextLayout's LineBreak (WordBoundary by default); inside a Canvas3D |
| `anchor` | `[number, number]` | Bevy's Anchor, as its Vec2: which point of the block sits at its Transform, [-0.5, -0.5] its bottom left to [0.5, 0.5] its top right ([0, 0], the centre, by default; Anchor::TOP_CENTER is [0, 0.5]) |
| `bounds` | `[number \| null, number \| null]` | TextBounds: the box its lines wrap in, [width, height] px (either null, unbounded), the anchor then the box's point; inside a Canvas3D |
| `shadow` | `Text2dShadow \| true` | Text2dShadow (true for its default); inside a Canvas3D |
| `children` | `ReactNode` | the text, and TextSpan runs |

Also takes the props of [`TextStyleProps`](#textstyleprops); [`TextRunProps`](#textrunprops) (`underline`, `strikethrough`, `underline_color`, `strikethrough_color`, `text_background_color`).

Used in: [shader/pipeline_constants](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/shader/pipeline_constants.tsx), [2d/text2d](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/text2d.tsx), [2d/sprite_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/sprite_slice.tsx), [2d/texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/texture_atlas.tsx), [2d/sprite_scale](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/sprite_scale.tsx), [2d/mesh2d_repeated_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/mesh2d_repeated_texture.tsx).

### `TextSpan`

A run of a <Text> in a style of its own (Bevy's defaults, not its Text's, where left out).

Bevy: `bevy_text::text::TextSpan`.

| Prop | Type | What it does |
| --- | --- | --- |
| `children` | `ReactNode` | the run's text |

Also takes the props of [`TextStyleProps`](#textstyleprops); [`TextRunProps`](#textrunprops) (`underline`, `strikethrough`, `underline_color`, `strikethrough_color`, `text_background_color`).

Used in: [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx), [ui/scroll_and_overflow/overflow_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_debug.tsx), [ui/text/text_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text_debug.tsx), [ui/text/text_background_colors](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text_background_colors.tsx), [ui/text/inline_image](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/inline_image.tsx), [ui/text/strikethrough_and_underline](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/strikethrough_and_underline.tsx), [ui/text/text](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text.tsx), [2d/text2d](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/text2d.tsx).

### `UiScale`

bevy_ui's UiScale: every UI length inside times `value` (2 draws a 16 px node 32 px), as CSS's zoom: laid out
and its text shaped at that size, in physical px, as Bevy lays out (a 3 px padding at 1.25 is 4 px, not 3 drawn 1.25
times as big). Text2d and the 2D and 3D parts aren't UI, so they stay as they are.

Bevy: `bevy_ui::UiScale`.

| Prop | Type | What it does |
| --- | --- | --- |
| `value` (required) | `number` | how many times as big the UI is drawn (UiScale) |
| `children` | `ReactNode` | the UI it scales |

Used in: [ui/images/ui_texture_slice_flip_and_tile](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice_flip_and_tile.tsx), [ui/scroll_and_overflow/scrollbars](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scrollbars.tsx), [ui/scroll_and_overflow/drag_to_scroll](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/drag_to_scroll.tsx).

### `UiTargetCamera`

The roots inside drawn by their own Camera2d (bevy_ui's UiTargetCamera): a layer over the whole window, stacked by
the camera's `order` as Bevy draws its cameras, lowest first. Cameras on one window share its main texture, which
only the first of them clears (bevy_render's ViewTarget, its first call), so the window's clear is the layered
<Canvas3D clearColor>'s and a later camera's own clear colour shows nowhere. Roots outside any are the default
camera's (the highest order).

Bevy: `bevy_ui::UiTargetCamera`.

| Prop | Type | What it does |
| --- | --- | --- |
| `order` | `number` | the camera, by its order, these roots draw with (0) |
| `children` | `ReactNode` | the UI roots that camera draws |

Used in: [ui/ui_target_camera](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_target_camera.tsx), [3d/split_screen](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/split_screen.tsx).

### `UiTexture`

The UI roots inside drawn into `target` instead of the window (a Camera2d with RenderTarget::Image, the roots'
UiTargetCamera): laid out in a box the image's size, see-through where they draw nothing, and nowhere in the frame
itself. A root's 100% is the image's size.

Bevy: `bevy_camera::RenderTarget::Image`.

| Prop | Type | What it does |
| --- | --- | --- |
| `target` (required) | `string` | the image's name, as a material or ImageNode uses it |
| `children` | `ReactNode` | the UI roots drawn into it |

Used in: [ui/render_ui_to_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/render_ui_to_texture.tsx).

### `Viewport`

The UI camera's viewport (bevy_camera's Viewport on the Camera2d the UI draws through): its roots lay out in this
region of the window, `physical_position` from its top left and `physical_size` big (px, at a scale factor of 1),
and are clipped to it.

Bevy: `bevy_camera::Viewport`.

| Prop | Type | What it does |
| --- | --- | --- |
| `physical_position` (required) | `[number, number]` | the region's top-left corner, px from the window's |
| `physical_size` (required) | `[number, number]` | the region's width and height, px |
| `children` | `ReactNode` | the cameras and UI drawn into the region |

Used in: [ui/relative_cursor_position](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/relative_cursor_position.tsx), [3d/split_screen](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/split_screen.tsx).

## Hooks

### `useRegion`

The Region this is rendered inside: its name and box in composition px.

```ts
function useRegion(): RegionBox
```

## Functions

### `actionsIn`

The actions handed in this frame, in order.

```ts
function actionsIn(input: FrameInput): { name: string; args: any; }[]
```

### `applyActions`

`state` with this frame's handed-in actions run on it, in order (an unknown name is skipped).

```ts
function applyActions<S>(actions: PanelActions<S>, state: S, input: FrameInput): S
```

### `feedIn`

What the app tells the panels, by name ("transcript", "object"); undefined until it has.

```ts
function feedIn<T = any>(input: FrameInput, what: string): T | undefined
```

### `percent`

Bevy's Val::Percent: of the parent's size on that axis (padding and margin: its width, as CSS)

```ts
function percent(n: number): string
```

Used in: [ui/images/image_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node.tsx), [ui/images/ui_texture_slice_flip_and_tile](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice_flip_and_tile.tsx), [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice.tsx), [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx), and 44 more.

### `proofBoxes`

Checks measured part boxes; returns one fault per problem, empty when all is well.

```ts
function proofBoxes({ width, height, face, parts }: ProofInput): string[]
```

### `px`

Bevy's Val::Px: logical pixels

```ts
function px(n: number): string
```

Used in: [ui/images/image_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node.tsx), [ui/images/ui_texture_slice_flip_and_tile](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice_flip_and_tile.tsx), [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice.tsx), [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx), and 57 more.

### `regionBand`

Works out the vertical band each name stands for, given perception and the composition size.

```ts
function regionBand(name: RegionName, width: number, height: number, perception: Perception, margin?: number): Band
```

### `targetImage`

An image the UI inside a <UiTexture> is drawn into, `width` x `height` px (bevy_image's Image::new_target_texture):
pass it to a material's texture field, e.g. StandardMaterial's base_color_texture. The name tells one target
from another (and from the composition's <Screen>s).

Bevy: `bevy_image::Image::new_target_texture`.

```ts
function targetImage(name: string, width: number, height: number): string
```

Used in: [ui/render_ui_to_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/render_ui_to_texture.tsx).

### `vh`

Bevy's Val::Vh: percent of the viewport's height

```ts
function vh(n: number): string
```

Used in: [ui/styling/stacked_gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/stacked_gradients.tsx), [ui/text/letter_spacing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/letter_spacing.tsx).

### `vmax`

Bevy's Val::VMax: percent of the viewport's larger side

```ts
function vmax(n: number): string
```

### `vmin`

Bevy's Val::VMin: percent of the viewport's smaller side

```ts
function vmin(n: number): string
```

### `vw`

Bevy's Val::Vw: percent of the viewport's width

```ts
function vw(n: number): string
```

Used in: [ui/text/letter_spacing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/letter_spacing.tsx).

## Values

### `ACTION_KEY`

The prefix the app puts on an action handed in as input: a key `action:NAME ARGS_JSON`, held for one frame.

### `AngularColorStop`

bevy_ui's AngularColorStop: a colour at `angle` radians round a conic gradient (spaced evenly when left out).

Bevy: `bevy_ui::gradients::AngularColorStop`.

Used in: [ui/styling/stacked_gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/stacked_gradients.tsx), [ui/styling/gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/gradients.tsx).

### `auto`

Bevy's Val::Auto

Used in: [ui/text/text_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text_input.tsx).

### `BorderRadius`

Bevy's BorderRadius: a Val (circular) or a CornerRadius (elliptical) for each corner (0 by default).

Used in: [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx), [ui/navigation/directional_navigation_overrides](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/navigation/directional_navigation_overrides.tsx), [ui/navigation/directional_navigation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/navigation/directional_navigation.tsx), [ui/scroll_and_overflow/scrollbars](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scrollbars.tsx), [ui/scroll_and_overflow/overflow_transform](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_transform.tsx), [ui/render_ui_to_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/render_ui_to_texture.tsx), [ui/ui_material](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_material.tsx), [ui/styling/gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/gradients.tsx), and 7 more.

### `BoxShadow`

bevy_ui's BoxShadow: drop shadows under a node, drawn back to front.

Bevy: `bevy_ui::BoxShadow`.

Used in: [ui/scroll_and_overflow/overflow_transform](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_transform.tsx), [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/standard_widgets](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/standard_widgets.tsx).

### `ColorStop`

bevy_ui's ColorStop: a colour at `point` along the gradient (spaced evenly when left out, as CSS's), and `hint`, where
between it and the next the colour is half way (0.5, linear, by default).

Bevy: `bevy_ui::gradients::ColorStop`.

Used in: [ui/styling/stacked_gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/stacked_gradients.tsx), [ui/styling/gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/gradients.tsx).

### `ConicGradient`

bevy_ui's ConicGradient: from `start` radians (0 up, clockwise) round `position`.

Bevy: `bevy_ui::gradients::ConicGradient`.

Used in: [ui/styling/stacked_gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/stacked_gradients.tsx), [ui/styling/gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/gradients.tsx).

### `CornerRadius`

Bevy's CornerRadius: a corner's x and y radii, an elliptical corner (a Val alone is a circular one).

Used in: [ui/styling/borders](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/borders.tsx).

### `FEED_KEY`

The prefix of the keys the app hands every panel for what it knows: `feed:transcript` (the conversation's last lines,
[{ who: "you" | "agent" | "tool" | "note", text, live }]), `feed:object` (the object on the stage: { name, kind }) and
`feed:stage` (what the object publishes with data-feed-publish, as a panel's publishes reach the object) and
`feed:player` (the stage's player: { frame, frames, rate, playing }; frames is 0 for a game, which runs on).
Handed in only on the frame it changes (and when the panel opens), so keep it in state:
`useGameState([], (s, input) => feedIn(input, "transcript") ?? s)`. A panel standing in for the terminal sends by
publishing { send: { n, text } } (a higher n is a new message); a panel drives the player by publishing
{ control: { n, do: "play" | "pause" | "seek", frame } } the same way.

### `FontAtlasSet`

bevy_text's FontAtlasSet, the glyph atlases Bevy's text is drawn from, for one face at one size:
`atlases(font, font_size, text)` is the pages a Text of `text` in the font file `font` at `font_size` px fills,
each glyph rasterized and packed by bevy_text's own code (natively) in the order its layout meets them. A text that
grows a character at a time fills them as its last version does.

Bevy: `bevy_text::FontAtlasSet`.

Used in: [ui/text/font_atlas_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/font_atlas_debug.tsx).

### `FontCx`

bevy_text's FontCx, the font collection Bevy's text resolves families in: `family_names()` lists the system's
families (natively, by fontique as Bevy's collection has them), sorted and each once.

Bevy: `bevy_text::FontCx`.

Used in: [ui/text/system_fonts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/system_fonts.tsx).

### `FontFeatureTag`

bevy_text's FontFeatureTag: the registered OpenType features' tags.

Bevy: `bevy_text::FontFeatureTag`.

Used in: [ui/text/text](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text.tsx).

### `FontSource`

bevy_text's FontSource: a font file (a string, staticFile("fonts/x.ttf")), { Family } by name, or { Generic }.

Bevy: `bevy_text::FontSource`.

Used in: [ui/text/system_fonts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/system_fonts.tsx), [ui/text/generic_font_families](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/generic_font_families.tsx), [ui/text/ime_support](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/ime_support.tsx).

### `FontVariationTag`

bevy_text's FontVariationTag: the registered axes' OpenType tags.

Bevy: `bevy_text::FontVariationTag`.

Used in: [ui/text/font_variations](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/font_variations.tsx).

### `FontWidth`

bevy_text's FontWidth: a face's width as a share of its normal one (CSS's font-stretch), for a Text's font_width.

Bevy: `bevy_text::FontWidth`.

Used in: [ui/text/font_query](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/font_query.tsx).

### `GridPlacement`

bevy_ui's GridPlacement: a start line, an end line (1-based, negative from the end) and a span (1 by default).

Bevy: `bevy_ui::GridPlacement`.

Used in: [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/anchor_layout](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/anchor_layout.tsx), [ui/ui_drag_and_drop](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_drag_and_drop.tsx), [ui/scroll_and_overflow/scrollbars](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scrollbars.tsx), [ui/scroll_and_overflow/drag_to_scroll](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/drag_to_scroll.tsx), [ui/text/multiple_text_inputs](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiple_text_inputs.tsx), [ui/text/generic_font_families](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/generic_font_families.tsx).

### `GridTrack`

bevy_ui's GridTrack: a track sized between its min and max sizing functions.

Bevy: `bevy_ui::GridTrack`.

Used in: [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/text/system_fonts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/system_fonts.tsx), [ui/text/multiple_text_inputs](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiple_text_inputs.tsx).

### `LinearGradient`

bevy_ui's LinearGradient: `angle` radians, 0 up and clockwise.

Bevy: `bevy_ui::gradients::LinearGradient`.

Used in: [ui/styling/stacked_gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/stacked_gradients.tsx), [ui/styling/gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/gradients.tsx).

### `Overflow`

Bevy's Overflow: each axis Visible, Clip, Hidden or Scroll.

Used in: [ui/scroll_and_overflow/overflow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow.tsx), [ui/scroll_and_overflow/scrollbars](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scrollbars.tsx), [ui/scroll_and_overflow/drag_to_scroll](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/drag_to_scroll.tsx), [ui/scroll_and_overflow/scroll](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scroll.tsx), [ui/scroll_and_overflow/overflow_clip_margin](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_clip_margin.tsx), [ui/text/multiple_text_inputs](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiple_text_inputs.tsx), [ui/text/text_wrap_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text_wrap_debug.tsx), [ui/widgets/headless_tabs](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/headless_tabs.tsx).

### `OverflowClipMargin`

Bevy's OverflowClipMargin: the box an overflow clip clips to, grown by a margin in px (the padding box by default).

Used in: [ui/scroll_and_overflow/overflow_clip_margin](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_clip_margin.tsx), [ui/text/multiple_text_inputs](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiple_text_inputs.tsx).

### `RadialGradient`

bevy_ui's RadialGradient: centred, ClosestCorner by default.

Bevy: `bevy_ui::gradients::RadialGradient`.

Used in: [ui/styling/stacked_gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/stacked_gradients.tsx), [ui/styling/gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/gradients.tsx).

### `RepeatedGridTrack`

bevy_ui's RepeatedGridTrack: tracks repeated.
bevy_ui's RepeatedGridTrack constructors: each GridTrack's, repeated `repetition` times.

Bevy: `bevy_ui::RepeatedGridTrack`.

Used in: [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/anchor_layout](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/anchor_layout.tsx), [ui/scroll_and_overflow/scrollbars](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scrollbars.tsx), [ui/scroll_and_overflow/drag_to_scroll](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/drag_to_scroll.tsx), [ui/scroll_and_overflow/overflow_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_debug.tsx), [ui/scroll_and_overflow/scroll](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/scroll.tsx), [ui/text/multiline_text_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiline_text_input.tsx), [ui/text/multiple_text_inputs](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/multiple_text_inputs.tsx), and 1 more.

### `Rot2`

bevy_math's Rot2: a rotation by its cosine and sine; positive turns a UI node clockwise.

Bevy: `bevy_math::Rot2`.

Used in: [ui/ui_transform](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_transform.tsx), [ui/navigation/directional_navigation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/navigation/directional_navigation.tsx), [ui/scroll_and_overflow/overflow_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_debug.tsx), [ui/scroll_and_overflow/overflow_transform](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_transform.tsx).

### `SPACE_KEY`

The prefix of the key the app holds on the stage for each panel that publishes (`data-feed-publish` on an element):
`space:PANEL JSON`, held while the value stands, so the object on the stage reads what the panels picked.

### `TextShadow`

bevy_ui's TextShadow.

Bevy: `bevy_ui::widget::text::TextShadow`.

Used in: [ui/text/text](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/text/text.tsx), [ui/widgets/standard_widgets](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/standard_widgets.tsx), [ui/widgets/button](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/button.tsx), [ui/widgets/standard_widgets_observers](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/standard_widgets_observers.tsx).

### `TextureAtlasLayout`

bevy_image's TextureAtlasLayout: a sheet's size and its cells' rects in px.

Bevy: `bevy_image::TextureAtlasLayout`.

Used in: [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_atlas](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas.tsx).

### `UiPosition`

bevy_ui's UiPosition: a point of the node, `anchor` from its centre (each axis -0.5 to 0.5 of its size), moved `x`
and `y` from there: right and down from a left, top or centre anchor, left and up from a right or bottom one.

Bevy: `bevy_ui::UiPosition`.

Used in: [ui/styling/stacked_gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/stacked_gradients.tsx), [ui/styling/gradients](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/gradients.tsx).

### `UiRect`

Bevy's UiRect: a Val for each side (0 by default).

Used in: [ui/images/image_node](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node.tsx), [ui/images/image_node_resizing](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/image_node_resizing.tsx), [ui/images/ui_texture_atlas_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_atlas_slice.tsx), [ui/images/ui_texture_slice](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/images/ui_texture_slice.tsx), [ui/layout/grid](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/grid.tsx), [ui/layout/ghost_nodes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/ghost_nodes.tsx), [ui/layout/flex_layout](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/flex_layout.tsx), [ui/layout/size_constraints](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/layout/size_constraints.tsx), and 43 more.

### `UiTransform`

bevy_ui's UiTransform: the node scaled, then turned, then moved, about its centre, after layout.

Bevy: `bevy_ui::ui_transform::UiTransform`.

Used in: [ui/ui_transform](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/ui_transform.tsx), [ui/navigation/directional_navigation](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/navigation/directional_navigation.tsx), [ui/scroll_and_overflow/overflow_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_debug.tsx), [ui/scroll_and_overflow/overflow_transform](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_transform.tsx).

### `Val2`

bevy_ui's Val2: a translation's x and y (a percent of the node's own width and height; auto is 0).

Bevy: `bevy_ui::ui_transform::Val2`.

Used in: [ui/scroll_and_overflow/overflow_debug](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/scroll_and_overflow/overflow_debug.tsx).

## Types

### `AbsoluteFillProps`

The props of [`AbsoluteFill`](#absolutefill), tabled there.

### `ControlOrientation`

bevy_ui_widgets' ControlOrientation.

Bevy: `bevy_ui_widgets::ControlOrientation`.

```ts
type ControlOrientation = "Horizontal" | "Vertical"
```

### `EditableTextProps`

The props of [`EditableText`](#editabletext), tabled there.

### `FontAtlas`

One page of bevy_text's glyph atlas: its texture (a PNG data: URL, which an ImageNode shows) and its size.

Bevy: `bevy_text::FontAtlas`.

| Field | Type | What it is |
| --- | --- | --- |
| `texture` (required) | `string` | Bevy: The texture where this font atlas is located |
| `width` (required) | `number` | the texture's width in px |
| `height` (required) | `number` | the texture's height in px |

### `GenericFontFamily`

bevy_text's GenericFontFamily.

Bevy: `bevy_text::GenericFontFamily`.

```ts
type GenericFontFamily = "Serif" | "SansSerif" | "Cursive" | "Fantasy" | "Monospace" | "SystemUi" | "UiSerif" | "UiSansSerif" | "UiMonospace" | "UiRounded" | "Emoji"
```

### `Gradient`

One of bevy_ui's Gradients.

Bevy: `bevy_ui::gradients::Gradient`.

```ts
type Gradient = | { kind: "Linear"; angle: number; stops: ColorStop[]; color_space?: InterpolationColorSpace } | { kind: "Radial"; position: UiPosition; shape: RadialGradientShape; stops: ColorStop[]; color_space?: InterpolationColorSpace } | { kind: "Conic"; start: number; position: UiPosition; stops: AngularCo…
```

### `GridAutoFlow`

bevy_ui's GridAutoFlow.

Bevy: `bevy_ui::GridAutoFlow`.

```ts
type GridAutoFlow = "Row" | "Column" | "RowDense" | "ColumnDense"
```

### `GridTrackRepetition`

bevy_ui's GridTrackRepetition: a count, or as many as fit (AutoFill), the empty ones collapsed (AutoFit).

Bevy: `bevy_ui::GridTrackRepetition`.

```ts
type GridTrackRepetition = number | "AutoFill" | "AutoFit"
```

### `IgnoreScroll`

bevy_ui's IgnoreScroll: the axes on which a node keeps its place while its parent scrolls (BVec2).

Bevy: `bevy_ui::IgnoreScroll`.

| Field | Type | What it is |
| --- | --- | --- |
| `x` (required) | `boolean` | it keeps its place while the parent scrolls across |
| `y` (required) | `boolean` | it keeps its place while the parent scrolls down |

### `ImageNodeProps`

The props of [`ImageNode`](#imagenode), tabled there.

### `InterpolationColorSpace`

bevy_ui's InterpolationColorSpace: Oklaba by default.

Bevy: `bevy_ui::gradients::InterpolationColorSpace`.

```ts
type InterpolationColorSpace = "Oklaba" | "Oklcha" | "OklchaLong" | "Srgba" | "LinearRgba" | "Hsla" | "HslaLong" | "Okhsla" | "OkhslaLong"
```

### `MaterialNodeProps`

The props of [`MaterialNode`](#materialnode), tabled there.

### `MaxTrackSizingFunction`

bevy_ui's MaxTrackSizingFunction: a MinTrackSizingFunction, a Fraction of the free space, or a fit-content limit.

Bevy: `bevy_ui::MaxTrackSizingFunction`.

```ts
type MaxTrackSizingFunction = MinTrackSizingFunction | { Fraction: number } | { FitContentPx: number } | { FitContentPercent: number }
```

### `MinTrackSizingFunction`

bevy_ui's MinTrackSizingFunction.

Bevy: `bevy_ui::MinTrackSizingFunction`.

```ts
type MinTrackSizingFunction = | "Auto" | "MinContent" | "MaxContent" | { Px: number } | { Percent: number } | { Rem: number } | { Vw: number } | { Vh: number } | { VMin: number } | { VMax: number }
```

### `NodeProps`

bevy_ui's Node: its layout fields as Bevy names them (each Bevy's default when left out), and the components that
draw it: background_color (BackgroundColor), border_color (BorderColor, every side), z_index (ZIndex).

Bevy: `bevy_ui::Node`.

### `PanelAction`

One action: what it does (for the agent), its arguments by name with a few words each, and what it does to the
panel's state.

| Field | Type | What it is |
| --- | --- | --- |
| `about` (required) | `string` | what the action does, for the agent |
| `args` | `Record<string, string>` | its arguments by name, a few words each |
| `run` (required) | `(state: S, args: A) => S` | the panel's state after it |

### `PanelActions`

```ts
type PanelActions = Record<string, PanelAction<S>>
```

### `Popover`

A popup placed beside its parent's box (inside its border) after layout, at the first of `positions` the window
(less `window_margin` px) hides least; absolutely positioned, as Bevy's position_popover makes it.

Bevy: `bevy_ui_widgets::popover::Popover`.

| Field | Type | What it is |
| --- | --- | --- |
| `positions` (required) | `PopoverPlacement[]` | Bevy: List of potential positions for the popover element relative to the parent. |
| `window_margin` | `number` | Bevy: Indicates how close to the window edge the popup is allowed to go. |

### `PopoverPlacement`

One of a Popover's placements beside its parent.

Bevy: `bevy_ui_widgets::popover::PopoverPlacement`.

| Field | Type | What it is |
| --- | --- | --- |
| `side` | `"Top" \| "Bottom" \| "Left" \| "Right"` | PopoverSide: Bottom by default |
| `align` | `"Start" \| "End" \| "Center"` | PopoverAlign: which edges line up along that side, Start by default |
| `gap` | `number` | the gap from the parent, in px (0 by default) |

### `ProofInput`

| Field | Type | What it is |
| --- | --- | --- |
| `width` (required) | `number` | the composition's width, px |
| `height` (required) | `number` | the composition's height, px |
| `face` | `Rect` | the face's box, when perception found one |
| `parts` (required) | `ProofPart[]` | the measured boxes of the parts to check |

### `ProofPart`

| Field | Type | What it is |
| --- | --- | --- |
| `name` (required) | `string` | the part's name, in the faults |
| `band` | `[number, number]` | the band the part must fit in, [top, bottom] in px |
| `x` (required) | `number` | the left edge, px |
| `y` (required) | `number` | the top edge, px |
| `w` (required) | `number` | width, px |
| `h` (required) | `number` | height, px |

### `RadialGradientShape`

bevy_ui's RadialGradientShape (ClosestCorner by default): Bevy's ClosestSide and FarthestSide are circles to the
nearest or farthest side, its ClosestCorner and FarthestCorner ellipses through the nearest or farthest sides on each
axis (CSS's closest-side and farthest-side ellipses), Circle a radius, Ellipse two.

Bevy: `bevy_ui::gradients::RadialGradientShape`.

```ts
type RadialGradientShape = "ClosestSide" | "FarthestSide" | "ClosestCorner" | "FarthestCorner" | { Circle: Val } | { Ellipse: [Val, Val] }
```

### `RegionBox`

The band a Region occupies, in composition px.

| Field | Type | What it is |
| --- | --- | --- |
| `name` (required) | `RegionName` | the band it took: top, bottom, center or full ("auto" resolves to top or bottom) |
| `x` (required) | `number` | the left edge, px |
| `y` (required) | `number` | the top edge, px |
| `w` (required) | `number` | width, px |
| `h` (required) | `number` | height, px |

### `RegionName`

```ts
type RegionName = "auto" | "top" | "bottom" | "center" | "full"
```

### `RegionProps`

The props of [`Region`](#region), tabled there.

### `ScrollbarProps`

The props of [`Scrollbar`](#scrollbar), tabled there.

### `ScrollbarThumbProps`

The props of [`ScrollbarThumb`](#scrollbarthumb), tabled there.

### `ShadowStyle`

bevy_ui's ShadowStyle: one drop shadow's colour, offset, spread and blur.

Bevy: `bevy_ui::ShadowStyle`.

| Field | Type | What it is |
| --- | --- | --- |
| `color` (required) | `unknown` | Bevy: The shadow's color |
| `x_offset` (required) | `Val` | Bevy: Horizontal offset |
| `y_offset` (required) | `Val` | Bevy: Vertical offset |
| `spread_radius` (required) | `Val` | Bevy: How much the shadow should spread outward. |
| `blur_radius` (required) | `Val` | Bevy: Blurriness of the shadow |

### `SliceScaleMode`

bevy_ui's SliceScaleMode: a side or the centre stretched, or tiled every slice x stretch_value px from its start

```ts
type SliceScaleMode = "Stretch" | { Tile: { stretch_value: number } }
```

### `Text2dProps`

The props of [`Text2d`](#text2d), tabled there.

### `TextCursorStyle`

bevy_text's TextCursorStyle (color: SLATE_700 by default).

Bevy: `bevy_text::TextCursorStyle`.

| Field | Type | What it is |
| --- | --- | --- |
| `color` | `Color` | Bevy: Color of the cursor |
| `selection_color` | `Color` | Bevy: Background color of selected text |
| `unfocused_selection_color` | `Color` | Bevy: Background color of unfocused selected text |
| `selected_text_color` | `Color` | Bevy: If some, overrides the color of selected text |
| `selection_radius` | `number` | Bevy: Corner radius of selection highlight rectangles, normalized relative to the selection height. |

### `TextProps`

### `TextRunProps`

What bevy_text draws with a section's own glyphs (a Text's string, not its spans; a TextSpan's run).

| Field | Type | What it is |
| --- | --- | --- |
| `underline` | `boolean` | Underline and Strikethrough: a line under or through the run, as thick as the font says |
| `strikethrough` | `boolean` | a line through the run |
| `underline_color` | `Color` | UnderlineColor and StrikethroughColor: the lines' colours (the text's colour by default) |
| `strikethrough_color` | `Color` | the strikethrough's colour (the text's by default) |
| `text_background_color` | `Color` | TextBackgroundColor: a box behind the run, line by line |

### `TextStyleProps`

bevy_text's TextFont, TextColor, TextLayout and LineHeight, on a Text or a TextSpan.

Bevy: `bevy_text::TextFont`.

| Field | Type | What it is |
| --- | --- | --- |
| `font` | `FontSource` | FontSource: a font file (staticFile("fonts/FiraSans-Bold.ttf")), FontSource.family(name) or a generic family (FontSource.serif()); Bevy's default font when left out |
| `font_size` | `number \| { Px: number } \| { Rem: number } \| { Vw: number } \| { Vh: number } \| { VMin: n…` | Bevy's FontSize: a number of px, or { Px }, { Rem } (of bevy_text's 20 px rem), { Vw }, { Vh }, { VMin }, { VMax } |
| `weight` | `number` | FontWeight: 100 to 900 (400 by default) |
| `style` | `"Normal" \| "Italic" \| "Oblique"` | Bevy: The slant style of a font face: normal, italic, or oblique. |
| `font_width` | `number` | TextFont's width (FontWidth): a share of the face's normal width, FontWidth.CONDENSED 0.75 (1 by default) |
| `color` | `Color` | TextColor: white by default |
| `line_height` | `{ Px: number } \| { RelativeToFont: number }` | LineHeight: { Px } or { RelativeToFont } (1.2 of the size by default) |
| `font_variations` | `Record<string, number>` | FontVariations: a variable font's axes by their four-letter tag, { [FontVariationTag.WEIGHT]: 300 } |
| `font_features` | `Record<string, number>` | FontFeatures: OpenType features by their four-letter tag, 1 to turn one on, { [FontFeatureTag.SMALL_CAPS]: 1 } |
| `letter_spacing` | `{ Px: number } \| { Rem: number }` | LetterSpacing: { Px } or { Rem } (of bevy_text's 20 px rem) after every glyph (none by default) |
| `font_smoothing` | `"AntiAliased" \| "None"` | TextFont's FontSmoothing: "None" draws the glyphs unantialiased (a <Text2d> inside a Canvas3D only) |

### `TextureSlicer`

bevy_sprite's TextureSlicer: the slices' insets in the picture's px, how the sides and centre scale (Stretch by
default), and the corners' most scale (1 by default)

| Field | Type | What it is |
| --- | --- | --- |
| `border` (required) | `{ left: number; right: number; top: number; bottom: number }` | the slices' insets, in the picture's px |
| `center_scale_mode` | `SliceScaleMode` | how the centre scales (Stretch) |
| `sides_scale_mode` | `SliceScaleMode` | how the sides scale (Stretch) |
| `max_corner_scale` | `number` | how much the corners may scale at most (1) |

### `UiDebugOptions`

bevy_ui's UiDebugOptions (and GlobalUiDebugOptions): which of a node's boxes to outline, how thick (px), in what
colour (each node its own by default), and whether a clipped or hidden node's outline shows. Every field Bevy's
default when left out.

Bevy: `bevy_ui_render::UiDebugOptions`.

| Field | Type | What it is |
| --- | --- | --- |
| `enabled` | `boolean` | Bevy: Set to true to enable the UI debug overlay |
| `outline_border_box` | `boolean` | Bevy: Show outlines for the border boxes of UI nodes |
| `outline_padding_box` | `boolean` | Bevy: Show outlines for the padding boxes of UI nodes |
| `outline_content_box` | `boolean` | Bevy: Show outlines for the content boxes of UI nodes |
| `line_width` | `number` | Bevy: Width of the overlay's lines in logical pixels |
| `line_color_override` | `Srgba` | one colour for every outline, instead of each node's (Hsla::sequential_dispersed of its index) |
| `show_clipped` | `boolean` | an outline not cut by its ancestors' overflow |

### `Val`

A bevy_ui Val as CSS: px(12) is "12px", percent(50) "50%", auto "auto".

```ts
type Val = string
```

<!-- generated:end -->
