---
title: "raea/feathers"
summary: "Bevy's feathers widgets: buttons, sliders, checkboxes, menus, inputs and colour pickers, in its dark theme."
order: 84
sources: ["sdk/raea/src/feathers.ts"]
---

Feathers is Bevy's widget set for tools and editors. Each widget here is one of Bevy's, drawn the way Bevy draws it:
buttons, tool buttons, checkboxes, radios, toggle switches, sliders, number and text inputs, selects, menus, list
views, scrollbars and colour pickers, in Bevy's dark theme and fonts. `UiTheme` and the theme components restyle
them.

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { AngleDegrees, caption, channel_range, context_color } from "raea/feathers"
```

`raea/feathers` exports 33 components, 2 hooks, 24 functions, 11 values, 27 types. Bevy's counterpart: `bevy_feathers, docs/control.feature FC0a`. Source: [sdk/raea/src/feathers.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/feathers.ts).

## Components

### `FeathersButton`

A feathers button: a ROW_HEIGHT node, 8 px each side, its corners 4 px, on the theme's button colour; its own Node
fields (flex_grow, a min_width) patch the button's, as a bsn! patch does.

Bevy: `bevy_feathers::controls::FeathersButton`.

Takes [`FeathersButtonProps`](#feathersbuttonprops)'s props.

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_counter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_counter.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx), [3d/split_screen](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/split_screen.tsx).

### `FeathersCheckbox`

A checkbox: an 18 px box (2 px border, 4 px corners) holding a check mark when checked, beside its caption.

Bevy: `bevy_feathers::controls::FeathersCheckbox`.

Takes [`FeathersToggleProps`](#featherstoggleprops)'s props.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersColorInput`

A colour input: a button showing its colour, opening a colour picker when clicked.

Bevy: `bevy_feathers::controls::FeathersColorInput`.

| Prop | Type | What it does |
| --- | --- | --- |
| `value` | `FeathersColor` | ColorInputValue: the colour it holds |
| `opaque_color_percentage` | `number` | how much of the swatch's right side shows the colour opaque, in percent |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersColorPlane`

A colour plane: at least 100 px high, 4 px padding, 5 px corners on COLOR_PLANE_BG, the plane filling its inside and
a 10 px ringed thumb at the value.

Bevy: `bevy_feathers::controls::FeathersColorPlane`.

| Prop | Type | What it does |
| --- | --- | --- |
| `plane` | `ColorPlane` | the two channels it spans, x then y (the FeathersColorPlane variant): RedGreen, RedBlue, GreenBlue, HueSaturation, HueLightness, OkhslHueSaturation or OkhslHueLightness |
| `value` | `[number, number, number]` | ColorPlaneValue: x and y the channels across and up (0 to 1, a hue in turns), z the fixed one |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersColorSlider`

A colour slider: a 16 px track of the channel's gradient over the alpha pattern, a 14 px ringed thumb at the value.

Bevy: `bevy_feathers::controls::FeathersColorSlider`.

| Prop | Type | What it does |
| --- | --- | --- |
| `value` | `number` | SliderValue: the channel's value, in its range |
| `channel` | `ColorChannel` | which colour channel it sets: Red, Green, Blue, Alpha, or HSL or Okhsl hue, saturation or lightness |
| `base_color` | `FeathersColor` | SliderBaseColor: the colour the track shows the channel of (white by default) |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersColorSwatch`

A colour swatch: ROW_HEIGHT high, at least as wide, its colour over the alpha pattern, the right
`opaque_color_percentage` of it the colour made opaque.

Bevy: `bevy_feathers::controls::FeathersColorSwatch`.

| Prop | Type | What it does |
| --- | --- | --- |
| `color` | `FeathersColor` | ColorSwatchValue: the colour shown over the checkerboard |
| `opaque_color_percentage` | `number` | how much of its right side shows the colour opaque, in percent |
| `corners` | `RoundedCorners` | which corners are rounded |
| `border_radius` | `number` | the corners' radius in px (5 by default), on the corners `corners` names |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerMove`, `onPointerUp`, `onPointerEnter`, `onPointerLeave`, `onPointerOver`, `onPointerOut`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersDisclosureToggle`

A disclosure toggle: a 12 px box holding a right chevron in BUTTON_TEXT, turned a quarter (pointing down) when
checked.

Bevy: `bevy_feathers::controls::FeathersDisclosureToggle`.

Also takes the props of [`NodeProps`](ui.md#nodeprops); [`FeathersToggleProps`](#featherstoggleprops) (`disabled`, `checked`); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerMove`, `onPointerUp`, `onPointerEnter`, `onPointerLeave`, `onPointerOver`, `onPointerOut`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersLazyMenu`

A menu whose popup is made when it opens (`popup`, here unused at rest): its button alone.

Bevy: `bevy_feathers::controls::FeathersLazyMenu`.

| Prop | Type | What it does |
| --- | --- | --- |
| `popup` | `() => ReactNode` | Bevy: Callback which constructs the popup |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersListRow`

A list row: at least ROW_HEIGHT, 8 px each side and 2 px above and below, on LISTROW_BG_SELECTED when selected, its
caption in Fira Sans 14 px.

Bevy: `bevy_feathers::controls::FeathersListRow`.

| Prop | Type | What it does |
| --- | --- | --- |
| `selected` | `boolean` | Selected |
| `disabled` | `boolean` | InteractionDisabled |
| `children` | `ReactNode` | its caption |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersListView`

A list box: its rows in a scroll area, a scrollbar track beside them.

Bevy: `bevy_feathers::controls::FeathersListView`.

| Prop | Type | What it does |
| --- | --- | --- |
| `children` | `ReactNode` | its FeathersListRows |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersMenu`

A menu: a ROW_HEIGHT node holding its button and popup.

Bevy: `bevy_feathers::controls::FeathersMenu`.

Takes [`NodeProps`](ui.md#nodeprops)'s props.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersMenuButton`

A menu's button: a Normal FeathersButton, its caption, then a spacer and a down chevron.

Bevy: `bevy_feathers::controls::FeathersMenuButton`.

Takes [`FeathersMenuButtonProps`](#feathersmenubuttonprops)'s props.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersMenuDivider`

A 1 px rule between menu items.

Bevy: `bevy_feathers::controls::FeathersMenuDivider`.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersMenuItem`

A menu item: a ROW_HEIGHT row, 8 px each side, its caption in MENUITEM_TEXT.

Bevy: `bevy_feathers::controls::FeathersMenuItem`.

Takes [`NodeProps`](ui.md#nodeprops)'s props.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersMenuPopup`

A menu's popup: a bordered column of items on the Floating level, hidden until the menu opens.

Bevy: `bevy_feathers::controls::FeathersMenuPopup`.

Takes [`NodeProps`](ui.md#nodeprops)'s props.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersMenuToolButton`

A menu's tool button: a FeathersToolButton, its caption, 2 px, then a down chevron.

Bevy: `bevy_feathers::controls::FeathersMenuToolButton`.

Takes [`FeathersMenuButtonProps`](#feathersmenubuttonprops)'s props.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersNumberInput`

A feathers number input: its value centred over a slider bar, in its units. Its own Node fields (flex_grow, a
max_width) patch the input's, as a bsn! patch does.

Bevy: `bevy_feathers::controls::FeathersNumberInput`.

| Prop | Type | What it does |
| --- | --- | --- |
| `value` (required) | `NumberInputValue` | NumberInputValue: the value shown |
| `soft_limit` | `NumberInputRange` | SoftLimit: the slider bar's range (no bar without one) |
| `hard_limit` | `NumberInputRange` | HardLimit: the value is held within it |
| `precision` | `number` | NumberInputPrecision: decimal places a drag rounds to |
| `step` | `number` | NumberInputStep: a drag's step |
| `wrap` | `NumberInputWrap` | NumberInputWrap (NoWrap by default) |
| `units` | `UnitsFormat` | NumberInputUnits: LengthMeters, TimeSeconds, AngleDegrees (Dimensionless by default) |
| `disabled` | `boolean` | InteractionDisabled |
| `sigil_color` | `ThemeToken` | the sigil's colour, the left border shown with a label (TEXT_INPUT_LABEL_BG by default) |
| `label_text` | `string` | the caption at its left, usually "X", "Y" or "Z" |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersRadio`

A radio button: an 18 px ring (2 px border) holding a 12 px dot when checked, beside its caption.

Bevy: `bevy_feathers::controls::FeathersRadio`.

Takes [`FeathersToggleProps`](#featherstoggleprops)'s props.

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersScrollbar`

A thin scrollbar: a track on SCROLLBAR_BG with 3 px corners, its thumb on SCROLLBAR_THUMB, at least 8 px long.

Bevy: `bevy_feathers::controls::FeathersScrollbar`.

| Prop | Type | What it does |
| --- | --- | --- |
| `target` (required) | `string` | the Name of the Node it scrolls |
| `orientation` | `ControlOrientation` | which way it scrolls: Vertical or Horizontal |
| `hidden` | `boolean` | hidden: the auto_hide decision, made by the list view (Bevy makes it after layout) |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

### `FeathersSelect`

A dropdown: a button showing the selected option, its list in a hidden popup.

Bevy: `bevy_feathers::controls::FeathersSelect`.

| Prop | Type | What it does |
| --- | --- | --- |
| `options` (required) | `ReactNode` | its options: FeathersListRows, one of them selected (list_rows_from_strings) |
| `corners` | `RoundedCorners` | which corners are rounded |
| `max_visible` | `number` | the popup's most rows (8 by default) |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersSlider`

A slider: a bar filled to its value, the value written on it.

Bevy: `bevy_feathers::controls::FeathersSlider`.

| Prop | Type | What it does |
| --- | --- | --- |
| `value` | `number` | SliderValue (min by default) |
| `min` | `number` | SliderRange's ends (0 to 1 by default) |
| `max` | `number` | SliderRange's top end (1) |
| `precision` | `number` | SliderPrecision: decimal places shown (0 by default) |
| `step` | `number` | SliderStep: a drag's or key's step |
| `disabled` | `boolean` | InteractionDisabled |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersTextInput`

A one-line text input in feathers' compact font, its colour the field's.

Bevy: `bevy_feathers::controls::FeathersTextInput`.

| Prop | Type | What it does |
| --- | --- | --- |
| `visible_width` | `number` | visible_width: its width in "0"s; it grows to fill the field when left out |
| `max_characters` | `number` | max_characters: how many characters typing may leave in it |

Also takes the props of [`NodeProps`](ui.md#nodeprops); [`TextStyleProps`](ui.md#textstyleprops); [`EditableTextProps`](ui.md#editabletextprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerMove`, `onPointerUp`, `onPointerEnter`, `onPointerLeave`, `onPointerOver`, `onPointerOut`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersTextInputContainer`

A text field's frame: ROW_HEIGHT high, 3 px each side, 4 px corners, growing to fill its row.

Bevy: `bevy_feathers::controls::FeathersTextInputContainer`.

| Prop | Type | What it does |
| --- | --- | --- |
| `children` | `ReactNode` | its FeathersTextInput, and anything beside it |

Also takes the props of [`NodeProps`](ui.md#nodeprops); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerUp`, `onPointerOver`, `onPointerOut`, `onPointerEnter`, `onPointerLeave`, `onPointerMove`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersToggleSwitch`

A toggle switch: a 32 x 18 track (2 px border, 5 px corners) and a slide filling its left half, or its right half
when checked.

Bevy: `bevy_feathers::controls::FeathersToggleSwitch`.

Also takes the props of [`NodeProps`](ui.md#nodeprops); [`FeathersToggleProps`](#featherstoggleprops) (`disabled`, `checked`); `PickProps` (`popover`, `onClick`, `onPointerDown`, `onPointerMove`, `onPointerUp`, `onPointerEnter`, `onPointerLeave`, `onPointerOver`, `onPointerOut`).

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `FeathersToolButton`

A FeathersButton for a toolbar: 4 px each side and at least ROW_HEIGHT wide.

Bevy: `bevy_feathers::controls::FeathersToolButton`.

Takes [`FeathersButtonProps`](#feathersbuttonprops)'s props.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `StandardUnitKind`

a flat table of units with scale factors, as a UnitsFormat.

Bevy: `bevy_feathers::controls::StandardUnitKind`.

Also takes the props of `String` (`toString`, `charAt`, `charCodeAt`, `concat`, `indexOf`, `lastIndexOf`, `localeCompare`, `match`, `replace`, `search`, `slice`, `split`, `substring`, `toLowerCase`, `toLocaleLowerCase`, `toUpperCase`, `toLocaleUpperCase`, `trim`, `length`, `substr`, `valueOf`, `codePointAt`, `includes`, `endsWith`, `normalize`, `repeat`, `startsWith`, `anchor`, `big`, `blink`, `bold`, `fixed`, `fontcolor`, `fontsize`, `italics`, `link`, `small`, `strike`, `sub`, `sup`, `padStart`, `padEnd`, `trimEnd`, `trimStart`, `trimLeft`, `trimRight`, `matchAll`, `replaceAll`, `at`, `isWellFormed`, `toWellFormed`, `__@iterator@155`).

### `ThemeBackgroundColor`

The element inside on the theme's colour for `token`.

Bevy: `bevy_feathers::theme::ThemeBackgroundColor`.

| Prop | Type | What it does |
| --- | --- | --- |
| `token` (required) | `ThemeToken` | the theme colour's token |
| `children` (required) | `ReactElement` | the one element it colours |

Used in: [ui/widgets/feathers_counter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_counter.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `ThemeBorderColor`

The element inside bordered in the theme's colour for `token`.

Bevy: `bevy_feathers::theme::ThemeBorderColor`.

| Prop | Type | What it does |
| --- | --- | --- |
| `token` (required) | `ThemeToken` | the theme colour's token |
| `children` (required) | `ReactElement` | the one element it borders |

### `ThemeContext`

the surface level the widgets inside sit on, which picks the theme's overrides for it.

Bevy: `bevy_feathers::theme::ThemeContext`.

| Prop | Type | What it does |
| --- | --- | --- |
| `level` (required) | `SurfaceLevel` | the surface level they sit on |
| `children` | `ReactNode` | the widgets on it |

### `ThemedText`

A Text with ThemedText: it takes the font and colour the feathers widget around it propagates.

Bevy: `bevy_feathers::theme::ThemedText`.

Takes [`TextProps`](ui.md#textprops)'s props.

Used in: [ui/widgets/feathers_counter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_counter.tsx).

### `ThemeTextColor`

The Text inside in the theme's colour for `token`.

Bevy: `bevy_feathers::theme::ThemeTextColor`.

| Prop | Type | What it does |
| --- | --- | --- |
| `token` (required) | `ThemeToken` | the theme colour's token |
| `children` (required) | `ReactElement` | the Text it colours |

### `UiTheme`

Bevy's UiTheme resource: the theme every feathers widget inside looks its colours up in (create_dark_theme()).

Bevy: `bevy_feathers::theme::UiTheme`.

| Prop | Type | What it does |
| --- | --- | --- |
| `theme` (required) | `ThemeProps` | the theme's colours by token |
| `children` | `ReactNode` | the widgets that use it |

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/virtual_keyboard](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/virtual_keyboard.tsx), [ui/widgets/feathers_counter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_counter.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx), [3d/split_screen](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/split_screen.tsx), [2d/2d_shapes](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/2d_shapes.tsx).

### `VirtualKeyboard`

A grid of FeathersButtons, one row of keys a row (4 px apart both ways), each key's caption a plain Text (Bevy's
default font, 20 px white: feathers' font reaches only a ThemedText), each key growing to share its row.
VirtualKeyPressed fires on a key under input.

Bevy: `bevy_feathers::controls::VirtualKeyboard`.

| Prop | Type | What it does |
| --- | --- | --- |
| `keys` (required) | `string[][]` | the keyboard's rows, each a row of key captions |

Used in: [ui/widgets/virtual_keyboard](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/virtual_keyboard.tsx).

## Hooks

### `useInheritedColor`

the text colour the feathers widget around the caller propagates (a ThemedIcon's tint), or none

```ts
function useInheritedColor(): Color | undefined
```

### `useThemeColor`

a token's colour in the UiTheme and surface level around the caller

```ts
function useThemeColor(token: ThemeToken): Srgba
```

## Functions

### `caption`

a ThemedText of `text`.

Bevy: `bevy_feathers::display::caption`.

```ts
function caption(text: string): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_counter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_counter.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx), [3d/split_screen](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/split_screen.tsx).

### `channel_range`

ColorChannel::range: 0 to 360 for a hue, 0 to 1 for the rest.

```ts
function channel_range(channel: ColorChannel): { start: number; end: number; }
```

### `context_color`

ThemeProps::context_color: a token's colour on a surface level, the level's override first, FUCHSIA when the theme
has none

```ts
function context_color(theme: ThemeProps, token: ThemeToken, level?: SurfaceLevel): Srgba
```

### `create_dark_theme`

bevy_feathers' dark theme.

Bevy: `bevy_feathers::dark_theme::create_dark_theme`.

```ts
function create_dark_theme(): ThemeProps
```

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/virtual_keyboard](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/virtual_keyboard.tsx), [ui/widgets/feathers_counter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_counter.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx), [3d/split_screen](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/split_screen.tsx).

### `flex_spacer`

A Node that grows to fill the row's free space.

Bevy: `bevy_feathers::containers::flex_spacer`.

```ts
function flex_spacer(): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `gradient_ends`

ColorChannel::gradient_ends: the slider track's colours at its start, middle and end, the channel at 0, half and
full with the base colour's other channels.

```ts
function gradient_ends(channel: ColorChannel, base: FeathersColor): [FeathersColor, FeathersColor, FeathersColor]
```

### `group`

A group: a bordered column (1 px, 4 px corners) of a group_header over a group_body, on the Highest level.

Bevy: `bevy_feathers::containers::group`.

```ts
function group(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `group_body`

A group's body: a column, 4 px apart, 6 px padding.

Bevy: `bevy_feathers::containers::group_body`.

```ts
function group_body(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `group_header`

A group's header: a row, 10 px each side, at least HEADER_HEIGHT.

Bevy: `bevy_feathers::containers::group_header`.

```ts
function group_header(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `icon`

One of feathers' icons, 14 px high, tinted by the text colour the widget around it propagates (white outside one),
as a ThemedIcon.

Bevy: `bevy_feathers::display::icon`.

```ts
function icon(i: FeathersIcon): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `icon_untinted`

One of feathers' icons, 14 px high, untinted.

Bevy: `bevy_feathers::display::icon_untinted`.

```ts
function icon_untinted(i: FeathersIcon): ReactElement<unknown, string | JSXElementConstructor<any>>
```

### `label`

`text` in Fira Sans 14 px, TEXT_MAIN.

Bevy: `bevy_feathers::display::label`.

```ts
function label(text: string): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `label_dim`

`text` in Fira Sans 14 px, TEXT_DIM.

Bevy: `bevy_feathers::display::label_dim`.

```ts
function label_dim(text: string): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `label_small`

`text` in Fira Sans 11 px, TEXT_MAIN.

Bevy: `bevy_feathers::display::label_small`.

```ts
function label_small(text: string): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `list_rows_from_strings`

FeathersListRows, one per label, captioned, the one at `selected` Selected.

Bevy: `bevy_feathers::controls::list_rows_from_strings`.

```ts
function list_rows_from_strings(labels: string[], selected?: number): ReactElement<unknown, string | JSXElementConstructor<any>>[]
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `pane`

A pane: a column of a pane_header over a pane_body.

Bevy: `bevy_feathers::containers::pane`.

```ts
function pane(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `pane_body`

A pane's body: a column, 4 px apart, 6 px padding, its bottom corners 4 px, on the Higher level.

Bevy: `bevy_feathers::containers::pane_body`.

```ts
function pane_body(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `pane_header`

A pane's header: a row of tools, 6 px padding and gaps, at least HEADER_HEIGHT, its top corners 4 px, a 1 px
border on three sides, on the Base level.

Bevy: `bevy_feathers::containers::pane_header`.

```ts
function pane_header(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `pane_header_divider`

A 1 px rule between a pane header's tools, reaching through the header's padding.

Bevy: `bevy_feathers::containers::pane_header_divider`.

```ts
function pane_header_divider(): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `slider_label`

update_slider_pos's label: Rust's {} on the f32, or {:.precision$} when that has no more decimals than precision

```ts
function slider_label(value: number, precision: number): string
```

### `subpane`

A sub-pane: a column of a subpane_header over a subpane_body.

Bevy: `bevy_feathers::containers::subpane`.

```ts
function subpane(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `subpane_body`

A sub-pane's body: a column, 4 px apart, 6 px padding, a 1 px border on three sides, its bottom corners 4 px, on the
Higher level.

Bevy: `bevy_feathers::containers::subpane_body`.

```ts
function subpane_body(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `subpane_header`

A sub-pane's header: a row, 10 px each side, at least HEADER_HEIGHT, its top corners 4 px, a 1 px border on three
sides, on the Highest level.

Bevy: `bevy_feathers::containers::subpane_header`.

```ts
function subpane_header(children?: ReactNode, node?: NodeProps): ReactElement<unknown, string | JSXElementConstructor<any>>
```

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `to_border_radius`

RoundedCorners::to_border_radius: `radius` px on the corners named, 0 on the rest

```ts
function to_border_radius(corners: RoundedCorners, radius: number): BorderRadius
```

## Values

### `AngleDegrees`

angles, canonical in radians, shown in degrees (°, or "d" while editing).

Bevy: `bevy_feathers::controls::AngleDegrees`.

Used in: [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx).

### `Dimensionless`

no units: the number alone.

Bevy: `bevy_feathers::controls::Dimensionless`.

### `fonts`

bevy_feathers' fonts, by family (the face by the TextFont's weight and style).

Bevy: `bevy_feathers::constants::fonts`.

### `icons`

bevy_feathers' icons.

Bevy: `bevy_feathers::constants::icons`.

Used in: [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `LengthMeters`

lengths, canonical and shown in metres.

Bevy: `bevy_feathers::controls::LengthMeters`.

Used in: [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx).

### `NumberInputRange`

an inclusive range of one number type.
NumberInputRange's variants, start..=end.

Bevy: `bevy_feathers::controls::NumberInputRange`, `bevy_feathers::controls::NumberInputRange`.

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `NumberInputValue`

a number input's value, in one of four types.
NumberInputValue's variants: an F32 is rounded to f32.

Bevy: `bevy_feathers::controls::NumberInputValue`, `bevy_feathers::controls::NumberInputValue`.

Used in: [ui/styling/box_shadow](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/styling/box_shadow.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

### `palette`

bevy_feathers' palette.

Bevy: `bevy_feathers::palette`.

### `size`

bevy_feathers' sizes.

Bevy: `bevy_feathers::constants::size`.

### `TimeSeconds`

times, canonical and shown in seconds.

Bevy: `bevy_feathers::controls::TimeSeconds`.

Used in: [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx).

### `tokens`

bevy_feathers' design tokens (ThemeToken ids).

Bevy: `bevy_feathers::tokens`.

Used in: [ui/widgets/feathers_counter](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_counter.tsx), [ui/widgets/feathers_number_input](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_number_input.tsx), [ui/widgets/feathers_gallery](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/ui/widgets/feathers_gallery.tsx).

## Types

### `ButtonVariant`

a button's look.

Bevy: `bevy_feathers::controls::ButtonVariant`.

```ts
type ButtonVariant = "Normal" | "Primary" | "Plain"
```

### `ColorChannel`

Which channel of a colour a FeathersColorSlider sets.

Bevy: `bevy_feathers::controls::ColorChannel`.

```ts
type ColorChannel = "Red" | "Green" | "Blue" | "HslHue" | "HslSaturation" | "HslLightness" | "Alpha" | "OkhslHue" | "OkhslSaturation" | "OkhslLightness"
```

### `FeathersButtonProps`

### `FeathersColor`

a colour a colour control takes

```ts
type FeathersColor = Srgba | LinearRgba | Hsla | Okhsla
```

### `FeathersColorInputProps`

The props of [`FeathersColorInput`](#featherscolorinput), tabled there.

### `FeathersColorPlaneProps`

The props of [`FeathersColorPlane`](#featherscolorplane), tabled there.

### `FeathersColorSliderProps`

The props of [`FeathersColorSlider`](#featherscolorslider), tabled there.

### `FeathersColorSwatchProps`

The props of [`FeathersColorSwatch`](#featherscolorswatch), tabled there.

### `FeathersIcon`

a feathers icon: its picture and size in px

| Field | Type | What it is |
| --- | --- | --- |
| `image` (required) | `string` | the icon's picture |
| `width` (required) | `number` | in px |
| `height` (required) | `number` | in px |

### `FeathersListRowProps`

The props of [`FeathersListRow`](#featherslistrow), tabled there.

### `FeathersListViewProps`

The props of [`FeathersListView`](#featherslistview), tabled there.

### `FeathersMenuButtonProps`

### `FeathersNumberInputProps`

The props of [`FeathersNumberInput`](#feathersnumberinput), tabled there.

### `FeathersScrollbarProps`

The props of [`FeathersScrollbar`](#feathersscrollbar), tabled there.

### `FeathersSelectProps`

The props of [`FeathersSelect`](#feathersselect), tabled there.

### `FeathersSliderProps`

The props of [`FeathersSlider`](#feathersslider), tabled there.

### `FeathersTextInputContainerProps`

The props of [`FeathersTextInputContainer`](#featherstextinputcontainer), tabled there.

### `FeathersTextInputProps`

The props of [`FeathersTextInput`](#featherstextinput), tabled there.

### `FeathersToggleProps`

### `NumberInputWrap`

whether a drag or step past a HardLimit wraps round.

Bevy: `bevy_feathers::controls::NumberInputWrap`.

```ts
type NumberInputWrap = "NoWrap" | "Wrap"
```

### `NumberKind`

the number type of a NumberInputValue or NumberInputRange

```ts
type NumberKind = "F32" | "F64" | "I32" | "I64"
```

### `RoundedCorners`

which corners are rounded.

Bevy: `bevy_feathers::rounded_corners::RoundedCorners`.

```ts
type RoundedCorners = "None" | "All" | "TopLeft" | "TopRight" | "BottomRight" | "BottomLeft" | "Top" | "Right" | "Bottom" | "Left"
```

### `StandardUnitItem`

one unit of a StandardUnitKind: its suffixes (the first canonical) and its size in the base unit.

Bevy: `bevy_feathers::controls::StandardUnitItem`.

| Field | Type | What it is |
| --- | --- | --- |
| `suffixes` (required) | `string[]` | Bevy: e.g. ["km"], or ["cm"], first entry is canonical |
| `scale` (required) | `number` | Bevy: multiply a value in this unit to get the base unit (meters, radians, kg...) |

### `SurfaceLevel`

the theme's surface levels, each a set of overrides of its semantic colours.

Bevy: `bevy_feathers::theme::SurfaceLevel`.

```ts
type SurfaceLevel = "Base" | "Higher" | "Highest" | "Floating"
```

### `ThemeProps`

a theme: its semantic colours, a surface level's overrides of them, and each token's semantic colour.

Bevy: `bevy_feathers::theme::ThemeProps`.

| Field | Type | What it is |
| --- | --- | --- |
| `semantic_base` (required) | `Record<string, Oklcha>` | Bevy: Map of semantic tokens to colors. |
| `semantic_overrides` (required) | `Partial<Record<SurfaceLevel, Record<string, Oklcha>>>` | Bevy: Map of semantic tokens + context to colors. |
| `token_assignments` (required) | `Partial<Record<ThemeToken, string>>` | Bevy: Map of design tokens to semantic tokens. |

### `ThemeToken`

a design token: one of `tokens`' ids

```ts
type ThemeToken = (typeof tokens)[keyof typeof tokens]
```

### `UnitsFormat`

formats a number input's value for display (or for editing).

Bevy: `bevy_feathers::controls::UnitsFormat`.

| Field | Type | What it is |
| --- | --- | --- |
| `id` (required) | `string` | the format's name |
| `format` (required) | `(value: NumberInputValue, editing: boolean) => string` | the value as text: for display, or for editing when \`editing\` is true |

<!-- generated:end -->
