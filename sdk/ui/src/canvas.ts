// Canvas 2D on the host (docs/ts-native.feature P15): a `<canvas>` element's getContext("2d") is a
// CanvasRenderingContext2D that records what's drawn as a display list, which the host draws with tiny-skia into the
// canvas's bitmap (systems/canvas), shown in the element's box as Chrome shows a canvas.
//
// The context keeps the state the spec gives it (the transform, styles, line and text settings, shadows, the clip, the
// save/restore stack) and resolves it into each drawing call, so the list is self-contained: paths are in the
// bitmap's pixels (each point transformed when it's added, arcs and ellipses as cubic Béziers), colours are RGBA
// numbers, and each call carries its transform, alpha, compositing, shadow and clip. The list since the bitmap was
// last cleared whole (clearRect over all of it, or a new width or height) goes to the host as the element's
// `__canvas2d` prop, at the next microtask after a drawing call.
//
// getImageData reads the pixels back synchronously (the host call canvas.read, which replays the list on the CPU).
// Not here (yet): putImageData / toDataURL, createPattern, filter, isPointInPath. measureText uses the advances the host gives for its fonts (`fonts`).
export { parseColor, parseFont, CanvasRenderingContext2D_, Path2D_, canvasClass, sniffSize, ImageData_, canvasGlobals } from "./canvas/all";


