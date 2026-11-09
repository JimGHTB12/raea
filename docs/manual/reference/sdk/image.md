---
title: "raea/image"
summary: "Pictures on the page and in the world, and texture atlases packed from them."
order: 45
sources: ["sdk/raea/src/image.ts"]
---

`<Img>` shows a picture on the page. `<ImagePlane>` and `<Plate>` put one into a 3D scene, as a card or as a
backdrop. `TextureAtlasBuilder` packs several pictures into one atlas with Bevy's own packer, for a `<Sprite>` to
show one cell of.

```tsx
import { staticFile } from "raea";
import { Img } from "raea/image";

function Logo() {
  return <Img src={staticFile("media/brand/logo.png")} style={{ width: 240 }} />;
}
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { ImagePlane, Img, Plate, TextureAtlasBuilder } from "raea/image"
```

`raea/image` exports 3 components, 1 class, 5 types. Bevy's counterpart: `bevy_image`. Source: [sdk/raea/src/image.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/image.ts).

## Components

### `ImagePlane`

A picture on a flat card in the scene, unlit (a poster, a screen, a sprite).

Bevy: `bevy_image::image::Image`.

Takes [`ImagePlaneProps`](3d.md#imageplaneprops)'s props.

### `Img`

An image; the renderer holds the frame until it has loaded and fails the render (naming src) if it can't.

Also takes the props of `ImgHTMLAttributes` (`alt`, `crossOrigin`, `decoding`, `fetchPriority`, `height`, `loading`, `referrerPolicy`, `sizes`, `src`, `srcSet`, `useMap`, `width`); `HTMLAttributes` (`defaultChecked`, `defaultValue`, `suppressContentEditableWarning`, `suppressHydrationWarning`, `accessKey`, `autoCapitalize`, `autoFocus`, `className`, `contentEditable`, `contextMenu`, `dir`, `draggable`, `enterKeyHint`, `hidden`, `id`, `lang`, `nonce`, `slot`, `spellCheck`, `style`, `tabIndex`, `title`, `translate`, `radioGroup`, `role`, `about`, `content`, `datatype`, `inlist`, `prefix`, `property`, `rel`, `resource`, `rev`, `typeof`, `vocab`, `autoCorrect`, `autoSave`, `color`, `itemProp`, `itemScope`, `itemType`, `itemID`, `itemRef`, `results`, `security`, `unselectable`, `popover`, `popoverTargetAction`, `popoverTarget`, `inert`, `inputMode`, `is`, `exportparts`, `part`); `AriaAttributes` (`aria-activedescendant`, `aria-atomic`, `aria-autocomplete`, `aria-braillelabel`, `aria-brailleroledescription`, `aria-busy`, `aria-checked`, `aria-colcount`, `aria-colindex`, `aria-colindextext`, `aria-colspan`, `aria-controls`, `aria-current`, `aria-describedby`, `aria-description`, `aria-details`, `aria-disabled`, `aria-dropeffect`, `aria-errormessage`, `aria-expanded`, `aria-flowto`, `aria-grabbed`, `aria-haspopup`, `aria-hidden`, `aria-invalid`, `aria-keyshortcuts`, `aria-label`, `aria-labelledby`, `aria-level`, `aria-live`, `aria-modal`, `aria-multiline`, `aria-multiselectable`, `aria-orientation`, `aria-owns`, `aria-placeholder`, `aria-posinset`, `aria-pressed`, `aria-readonly`, `aria-relevant`, `aria-required`, `aria-roledescription`, `aria-rowcount`, `aria-rowindex`, `aria-rowindextext`, `aria-rowspan`, `aria-selected`, `aria-setsize`, `aria-sort`, `aria-valuemax`, `aria-valuemin`, `aria-valuenow`, `aria-valuetext`); `DOMAttributes` (`children`, `dangerouslySetInnerHTML`, `onCopy`, `onCopyCapture`, `onCut`, `onCutCapture`, `onPaste`, `onPasteCapture`, `onCompositionEnd`, `onCompositionEndCapture`, `onCompositionStart`, `onCompositionStartCapture`, `onCompositionUpdate`, `onCompositionUpdateCapture`, `onFocus`, `onFocusCapture`, `onBlur`, `onBlurCapture`, `onChange`, `onChangeCapture`, `onBeforeInput`, `onBeforeInputCapture`, `onInput`, `onInputCapture`, `onReset`, `onResetCapture`, `onSubmit`, `onSubmitCapture`, `onInvalid`, `onInvalidCapture`, `onLoad`, `onLoadCapture`, `onError`, `onErrorCapture`, `onKeyDown`, `onKeyDownCapture`, `onKeyPress`, `onKeyPressCapture`, `onKeyUp`, `onKeyUpCapture`, `onAbort`, `onAbortCapture`, `onCanPlay`, `onCanPlayCapture`, `onCanPlayThrough`, `onCanPlayThroughCapture`, `onDurationChange`, `onDurationChangeCapture`, `onEmptied`, `onEmptiedCapture`, `onEncrypted`, `onEncryptedCapture`, `onEnded`, `onEndedCapture`, `onLoadedData`, `onLoadedDataCapture`, `onLoadedMetadata`, `onLoadedMetadataCapture`, `onLoadStart`, `onLoadStartCapture`, `onPause`, `onPauseCapture`, `onPlay`, `onPlayCapture`, `onPlaying`, `onPlayingCapture`, `onProgress`, `onProgressCapture`, `onRateChange`, `onRateChangeCapture`, `onSeeked`, `onSeekedCapture`, `onSeeking`, `onSeekingCapture`, `onStalled`, `onStalledCapture`, `onSuspend`, `onSuspendCapture`, `onTimeUpdate`, `onTimeUpdateCapture`, `onVolumeChange`, `onVolumeChangeCapture`, `onWaiting`, `onWaitingCapture`, `onAuxClick`, `onAuxClickCapture`, `onClick`, `onClickCapture`, `onContextMenu`, `onContextMenuCapture`, `onDoubleClick`, `onDoubleClickCapture`, `onDrag`, `onDragCapture`, `onDragEnd`, `onDragEndCapture`, `onDragEnter`, `onDragEnterCapture`, `onDragExit`, `onDragExitCapture`, `onDragLeave`, `onDragLeaveCapture`, `onDragOver`, `onDragOverCapture`, `onDragStart`, `onDragStartCapture`, `onDrop`, `onDropCapture`, `onMouseDown`, `onMouseDownCapture`, `onMouseEnter`, `onMouseLeave`, `onMouseMove`, `onMouseMoveCapture`, `onMouseOut`, `onMouseOutCapture`, `onMouseOver`, `onMouseOverCapture`, `onMouseUp`, `onMouseUpCapture`, `onSelect`, `onSelectCapture`, `onTouchCancel`, `onTouchCancelCapture`, `onTouchEnd`, `onTouchEndCapture`, `onTouchMove`, `onTouchMoveCapture`, `onTouchStart`, `onTouchStartCapture`, `onPointerDown`, `onPointerDownCapture`, `onPointerMove`, `onPointerMoveCapture`, `onPointerUp`, `onPointerUpCapture`, `onPointerCancel`, `onPointerCancelCapture`, `onPointerEnter`, `onPointerLeave`, `onPointerOver`, `onPointerOverCapture`, `onPointerOut`, `onPointerOutCapture`, `onGotPointerCapture`, `onGotPointerCaptureCapture`, `onLostPointerCapture`, `onLostPointerCaptureCapture`, `onScroll`, `onScrollCapture`, `onScrollEnd`, `onScrollEndCapture`, `onWheel`, `onWheelCapture`, `onAnimationStart`, `onAnimationStartCapture`, `onAnimationEnd`, `onAnimationEndCapture`, `onAnimationIteration`, `onAnimationIterationCapture`, `onToggle`, `onBeforeToggle`, `onTransitionCancel`, `onTransitionCancelCapture`, `onTransitionEnd`, `onTransitionEndCapture`, `onTransitionRun`, `onTransitionRunCapture`, `onTransitionStart`, `onTransitionStartCapture`).

### `Plate`

Footage behind the 3D: the clip's frame filling the camera's whole view (its `view` when it has one), on the far
side of everything, unlit. Give the scene `<ToneMapping mode="none" />` to keep its colours exact. <Matchmove> is
the usual way in.

Takes [`PlateProps`](3d.md#plateprops)'s props.

## Classes

### `TextureAtlasBuilder`

Pictures packed into one atlas, as Bevy's TextureAtlasBuilder: `padding([x, y])` px around each, `addTexture` in
order, then `build({ sampler })` (the atlas's own sampler, Bevy's default otherwise).

Bevy: `bevy_image::texture_atlas_builder::TextureAtlasBuilder`, `bevy_image::texture_atlas::TextureAtlasSources`.

| Member | What it does |
| --- | --- |
| `.padding(p: [number, number]): TextureAtlasBuilder` | – |
| `.addTexture(src: string): TextureAtlasBuilder` | – |
| `.build({ sampler }?: { sampler?: SamplerSpec \| undefined; }): BuiltAtlas` | – |

## Types

### `AtlasSprite`

What a <Sprite> takes to show a built atlas: the whole of it, or one of its pictures (`atlasPick`).

| Field | Type | What it is |
| --- | --- | --- |
| `src` (required) | `string` | – |
| `sampler` | `SamplerSpec` | – |
| `atlasPick` | `number` | – |

### `BuiltAtlas`

A built atlas: `image` the whole atlas as a sprite's picture, `sources.handle(texture)` one picture of it
(TextureAtlasSources::handle).

| Field | Type | What it is |
| --- | --- | --- |
| `image` (required) | `AtlasSprite` | – |
| `sources` (required) | `{ handle(texture: string): AtlasSprite }` | – |

### `ImagePlaneProps`

| Field | Type | What it is |
| --- | --- | --- |
| `src` (required) | `string` | a picture in the media folder: staticFile("poster.png") |
| `name` | `string` | – |
| `width` | `number` | width in scene units (1 by default); height follows the picture unless given |
| `height` | `number` | – |
| `position` | `Vec3` | – |
| `rotation` | `Vec3` | radians |
| `scale` | `number \| Vec3` | – |
| `quaternion` | `[number, number, number, number]` | a turn as a quaternion [x, y, z, w], instead of `rotation` (a card turned to face a point) |
| `shadow` | `boolean` | false: the card casts no shadow (a cursor or a UI floating in the scene); true by default |
| `cutout` | `boolean` | the picture cut out at half alpha instead of blended: its clear parts are no surface at all (no fog on them, no sorting) and its shadow is the picture's shape, not the card's. For hard-edged pictures: a baked cloud, a sprite |
| `grid` | `[number, number]` | a sprite sheet: the picture is `[columns, rows]` frames of equal size, read left to right, top to bottom, and the card shows one cell (its height follows one cell's shape). A texture atlas of a regular grid |
| `frame` | `number` | with `grid`: which cell shows (0 is the top left; whole numbers, wrapping); animate it to play the sheet, as Bevy's TextureAtlas index does: frame={Math.floor(f / 3)} |

### `ImgProps`

The props of [`Img`](#img), tabled there.

### `PlateProps`

| Field | Type | What it is |
| --- | --- | --- |
| `src` (required) | `string` | a video in the media folder: staticFile("clip.mp4") |
| `name` | `string` | – |
| `time` | `number` | the clip's time on this frame, in seconds (by default the Sequence's: frame / fps) |
| `k1` | `number` | the lens's radial distortion (SIMPLE_RADIAL's k1, from a solve): the plate is drawn undistorted, so the footage's straight lines are straight where the 3D's are |

<!-- generated:end -->
